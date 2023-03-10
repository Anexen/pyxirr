use std::{error::Error, fmt};

use crate::conversions::float_or_none;
use ndarray::{ArrayD, ArrayViewD, Axis, CowArray, IxDyn};
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyList};

/// An error returned when the payments do not contain both negative and positive payments.
#[derive(Debug)]
pub struct BroadcastingError(String);

impl BroadcastingError {
    pub fn new(shapes: &[&[usize]]) -> Self {
        Self(format!("{:?}", shapes))
    }
}

impl fmt::Display for BroadcastingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BroadcastingError {}

pub fn broadcast_shapes(shapes: &[&[usize]]) -> Option<Vec<usize>> {
    /* Discover the broadcast number of dimensions */
    let nd = shapes.iter().map(|s| s.len()).max()?;
    let mut result = vec![0; nd];

    /* Discover the broadcast shape in each dimension */
    for i in 0..nd {
        result[i] = 1;
        for s in shapes.iter() {
            /* This prepends 1 to shapes not already equal to nd */
            if i + s.len() >= nd {
                let k = i + s.len() - nd;
                let tmp = s[k];
                if tmp == 1 {
                    continue;
                }
                if result[i] == 1 {
                    result[i] = tmp;
                } else if result[i] != tmp {
                    return None;
                }
            }
        }
    }

    Some(result)
}

#[macro_export]
macro_rules! broadcast_together {
    ($($a:expr),*) => {
        {
            let _a = &[$($a.shape(),)*];

            match crate::broadcasting::broadcast_shapes(_a) {
                Some(shape) => Ok(( $($a.broadcast(shape.clone()).unwrap(),)*)),
                None => Err(BroadcastingError::new(_a))
            }
        }
    };
}

pub fn pylist_to_arrayd<'p, T>(pylist: &'p PyList) -> PyResult<ArrayD<T>>
where
    T: FromPyObject<'p>,
{
    let dims = get_pylist_dims(pylist)?;
    let mut flat_list = Vec::new();
    flatten_pylist(pylist, &mut flat_list)?;
    let arr = ArrayD::from_shape_vec(IxDyn(&dims), flat_list).unwrap();
    Ok(arr)
}

pub fn arrayd_to_pylist<'a>(py: Python<'a>, array: ArrayViewD<'_, f64>) -> PyResult<&'a PyList> {
    let list = PyList::empty(py);
    if array.ndim() == 1 {
        for &x in array {
            list.append(float_or_none(x).to_object(py))?;
        }
    } else {
        for subarray in array.axis_iter(Axis(0)) {
            let sublist = arrayd_to_pylist(py, subarray)?;
            list.append(sublist)?;
        }
    }
    Ok(list)
}

fn get_pylist_dims(pylist: &PyList) -> PyResult<Vec<usize>> {
    let mut dims = Vec::new();
    let mut sublist = pylist;

    loop {
        dims.push(sublist.len());
        if let Ok(new_sublist) = sublist.get_item(0) {
            if let Ok(s) = new_sublist.downcast::<PyList>() {
                sublist = s;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    Ok(dims)
}

fn flatten_pylist<'p, T>(pylist: &'p PyList, flat_list: &mut Vec<T>) -> PyResult<()>
where
    T: FromPyObject<'p>,
{
    for item in pylist.iter() {
        match item.extract::<T>() {
            Ok(val) => flat_list.push(val),
            Err(_) => {
                let sublist = item.extract::<&PyList>()?;
                flatten_pylist(&sublist, flat_list)?;
            }
        }
    }
    Ok(())
}

pub enum Arg<'p, T> {
    Scalar(T),
    Array(CowArray<'p, T, IxDyn>),
}

impl<'p, T> Arg<'p, T>
where
    T: Clone,
{
    pub fn to_arrayd(self) -> CowArray<'p, T, IxDyn> {
        self.into()
    }
}

impl<'p> FromPyObject<'p> for Arg<'p, f64> {
    fn extract(obj: &'p PyAny) -> PyResult<Self> {
        if let Ok(value) = obj.extract::<f64>() {
            return Ok(Arg::Scalar(value));
        };

        if let Ok(py_list) = obj.downcast::<PyList>() {
            let arr = pylist_to_arrayd(py_list)?;
            return Ok(Arg::Array(CowArray::from(arr)));
        }

        if let Ok(a) = obj.downcast::<numpy::PyArrayDyn<i64>>() {
            let arr = a.cast::<f64>(false)?.to_owned_array();
            return Ok(Arg::Array(CowArray::from(arr)));
        }

        if let Ok(a) = obj.downcast::<numpy::PyArrayDyn<f64>>() {
            let arr = unsafe { a.as_array() };
            return Ok(Arg::Array(CowArray::from(arr)));
        }

        Err(PyTypeError::new_err(""))
    }
}

impl<'p> FromPyObject<'p> for Arg<'p, bool> {
    fn extract(obj: &'p PyAny) -> PyResult<Self> {
        if let Ok(value) = obj.extract::<bool>() {
            return Ok(Arg::Scalar(value));
        };

        if let Ok(py_list) = obj.downcast::<PyList>() {
            let arr = pylist_to_arrayd(py_list)?;
            return Ok(Arg::Array(CowArray::from(arr)));
        }

        if let Ok(a) = obj.downcast::<numpy::PyArrayDyn<bool>>() {
            let arr = unsafe { a.as_array() };
            return Ok(Arg::Array(CowArray::from(arr)));
        }

        Err(PyTypeError::new_err(""))
    }
}

impl IntoPy<PyObject> for Arg<'_, f64> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

impl ToPyObject for Arg<'_, f64> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        // broadcasting::arrayd_to_pylist(py, result.view()).map(|x| x.into())
        // Ok(numpy::ToPyArray::to_pyarray(&result, py).to_object(py))
        match self {
            Arg::Scalar(s) => float_or_none(*s).into_py(py),
            Arg::Array(s) => match arrayd_to_pylist(py, s.view()) {
                Ok(py_list) => py_list.into_py(py),
                Err(err) => err.into_py(py),
            },
        }
    }
}

impl<'p, T> From<Arg<'p, T>> for CowArray<'p, T, IxDyn>
where
    T: Clone,
{
    fn from(arg: Arg<'p, T>) -> Self {
        match arg {
            Arg::Scalar(value) => CowArray::from(ndarray::arr1(&[value]).into_dyn()),
            Arg::Array(a) => a,
        }
    }
}

impl<T> From<ArrayD<T>> for Arg<'_, T> {
    fn from(arr: ArrayD<T>) -> Self {
        Arg::Array(CowArray::from(arr))
    }
}

// use numpy::npyffi::PY_ARRAY_API;
// use std::ffi::c_int;

// Python::with_gil(|py| {
//     let dt = numpy::dtype::<f64>(py);

//     let a_ptr = unsafe {
//         PY_ARRAY_API.PyArray_FromAny(
//             py,
//             obj.as_ptr(),
//             dt.as_dtype_ptr(),
//             0 as c_int,
//             0 as c_int,
//             0 as c_int,
//             std::ptr::null_mut(),
//         )
//     };

//     let pa: Py<PyArrayDyn<f64>> = unsafe { Py::from_borrowed_ptr(py, a_ptr) };
//     pa.as_ref(py).to_owned_array()

// impl<'p> Arg<'p> {
//     pub fn try_to_arrayd(&'p self) -> PyResult<CowArray<'p, f64, IxDyn>> {
//         match self {
//             Arg::Scalar(value) => {
//                 let arr = ndarray::arr1(&[*value]).into_dyn();
//                 Ok(CowArray::from(arr))
//             }
//             Arg::Any(obj) => {
//                 if let Ok(py_list) = obj.downcast::<PyList>() {
//                     return pylist_to_arrayd(py_list).map(CowArray::from);
//                 }

//                 // if !numpy::get_array_module().is_ok() {
//                 //     unimplemented!();
//                 // }

//                 if let Ok(a) = obj.downcast::<numpy::PyArrayDyn<i64>>() {
//                     return a.cast::<f64>(false).map(|p| CowArray::from(p.to_owned_array()));
//                 }

//                 if let Ok(a) = obj.downcast::<numpy::PyArrayDyn<f64>>() {
//                     return Ok(CowArray::from(unsafe { a.as_array() }));
//                 }

//                 Err(PyNotImplementedError::new_err(""))
//             }
//         }
//     }
// }

// trait ToCowArrayViewD {
//     fn to_cowarray_view_d<'p>(&'p self) -> Option<CowArray<'p, f64, IxDyn>>;
// }

// impl ToCowArrayViewD for f64 {
//     fn to_cowarray_view_d<'p>(&'p self) -> Option<CowArray<'p, f64, IxDyn>> {
//         let array = ndarray::arr1(&[*self]).into_dyn();
//         Some(CowArray::from(array))
//     }
// }

// impl ToCowArrayViewD for &PyList {
//     fn to_cowarray_view_d<'p>(&'p self) -> Option<CowArray<'p, f64, IxDyn>> {
//         pylist_to_arrayd(self).ok().map(CowArray::from)
//     }
// }

// impl<'p> ToCowArrayViewD<'p> for &'p PyArrayDyn<i64> {
//     fn to_cowarray_view_d(&'p self) -> Option<CowArray<'p, f64, IxDyn>> {
//         let py_array_f64 = self.cast::<f64>(false).ok()?;
//         let array = unsafe { py_array_f64.as_array().into_dyn() };
//         Some(CowArray::from(array))
//     }
// }

// impl<'p> ToCowArrayViewD<'p> for &'p PyArrayDyn<f64> {
//     fn to_cowarray_view_d(&'p self) -> Option<CowArray<'p, f64, IxDyn>> {
//         let array = unsafe { self.as_array().into_dyn() };
//         Some(CowArray::from(array))
//     }
// }

// impl ToCowArrayViewD for &PyAny {
//     fn to_cowarray_view_d<'p>(&'p self) -> Option<CowArray<'p, f64, IxDyn>> {
//         // if let Ok(value) = self.extract::<f64>() {
//         //     return value.to_cowarray_view_d();
//         // };

//         if let Ok(value) = self.downcast::<PyList>() {
//             return value.to_cowarray_view_d();
//         };

//         None
//     }
// }

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_broadcast_shapes() {
        assert_eq!(broadcast_shapes(&[&[3_usize, 2], &[2, 1]]), None);
        assert_eq!(broadcast_shapes(&[&[1_usize, 2], &[3, 1], &[3, 2]]), Some(vec![3_usize, 2]));
        assert_eq!(
            broadcast_shapes(&[&[6_usize, 7], &[5, 6, 1], &[7], &[5, 1, 7]]),
            Some(vec![5, 6, 7])
        );
    }

    // #[rstest]
    // fn test_cow_array() {
    //     Python::with_gil(|py| {
    //         let pyarray = PyArray1::arange(py, 1i64, 5, 1).to_dyn();
    //         let array = pyarray.to_cowarray_view_d().unwrap();
    //         assert_eq!(array.sum(), 10.0);
    //     });
    // }
}
