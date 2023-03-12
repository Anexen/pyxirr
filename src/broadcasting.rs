use std::{error::Error, fmt};

use crate::conversions::float_or_none;
use ndarray::{ArrayD, ArrayViewD, Axis, CowArray, IxDyn};
use numpy::{npyffi, Element, PyArrayDyn, PY_ARRAY_API};
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyList, AsPyPointer};

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
    let ndim = shapes.iter().map(|s| s.len()).max()?;
    let mut result = vec![0; ndim];

    /* Discover the broadcast shape in each dimension */
    for i in 0..ndim {
        result[i] = 1;
        for s in shapes.iter() {
            /* This prepends 1 to shapes not already equal to ndim */
            if i + s.len() >= ndim {
                let k = i + s.len() - ndim;
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
    NumpyArray(&'p PyArrayDyn<T>),
}

impl<'p, T> Arg<'p, T>
where
    T: Clone + numpy::Element,
{
    pub fn to_arrayd(self) -> CowArray<'p, T, IxDyn> {
        self.into()
    }
}

pub fn pyarray_cast<'p, U: Element>(ob: &'p PyAny) -> PyResult<&'p PyArrayDyn<U>> {
    let ptr = unsafe {
        PY_ARRAY_API.PyArray_CastToType(
            ob.py(),
            ob.as_ptr() as _,
            U::get_dtype(ob.py()).into_dtype_ptr(),
            0,
        )
    };
    if !ptr.is_null() {
        Ok(unsafe { PyArrayDyn::<U>::from_owned_ptr(ob.py(), ptr) })
    } else {
        Err(PyErr::fetch(ob.py()))
    }
}

impl<'p> FromPyObject<'p> for Arg<'p, f64> {
    fn extract(ob: &'p PyAny) -> PyResult<Self> {
        if let Ok(value) = ob.extract::<f64>() {
            return Ok(Arg::Scalar(value));
        };

        if let Ok(py_list) = ob.downcast::<PyList>() {
            let arr = pylist_to_arrayd(py_list)?;
            return Ok(Arg::Array(CowArray::from(arr)));
        }
        if let Ok(a) = ob.downcast::<numpy::PyArrayDyn<f64>>() {
            return Ok(Arg::NumpyArray(a));
            // let arr = unsafe { a.as_array() };
            // return Ok(Arg::Array(CowArray::from(arr)));
        }

        if unsafe { npyffi::PyArray_Check(ob.py(), ob.as_ptr()) } == 1 {
            let a = pyarray_cast::<f64>(ob)?;
            return Ok(Arg::NumpyArray(a));
        }

        Err(PyTypeError::new_err(""))
    }
}

impl<'p> FromPyObject<'p> for Arg<'p, bool> {
    fn extract(ob: &'p PyAny) -> PyResult<Self> {
        if let Ok(value) = ob.extract::<bool>() {
            return Ok(Arg::Scalar(value));
        };

        if let Ok(py_list) = ob.downcast::<PyList>() {
            let arr = pylist_to_arrayd(py_list)?;
            return Ok(Arg::Array(CowArray::from(arr)));
        }

        if let Ok(a) = ob.downcast::<numpy::PyArrayDyn<bool>>() {
            return Ok(Arg::NumpyArray(a));
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
            Arg::Array(a) => match arrayd_to_pylist(py, a.view()) {
                Ok(py_list) => py_list.into_py(py),
                Err(err) => err.into_py(py),
            },
            Arg::NumpyArray(a) => a.into_py(py),
        }
    }
}

impl<'p, T> From<Arg<'p, T>> for CowArray<'p, T, IxDyn>
where
    T: Clone + numpy::Element,
{
    fn from(arg: Arg<'p, T>) -> Self {
        match arg {
            Arg::Scalar(value) => CowArray::from(ndarray::arr1(&[value]).into_dyn()),
            Arg::Array(a) => a,
            Arg::NumpyArray(a) => CowArray::from(unsafe { a.as_array() }),
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
