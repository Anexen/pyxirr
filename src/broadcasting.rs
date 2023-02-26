use std::{error::Error, fmt};

use ndarray::{ArrayD, ArrayViewD, Axis, IxDyn};
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

pub fn pylist_to_arrayd(pylist: &PyList) -> PyResult<ArrayD<f64>> {
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
            list.append(x.to_object(py))?;
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

fn flatten_pylist(pylist: &PyList, flat_list: &mut Vec<f64>) -> PyResult<()> {
    for item in pylist.iter() {
        match item.extract::<f64>() {
            Ok(val) => flat_list.push(val),
            Err(_) => {
                let sublist = item.extract::<&PyList>()?;
                flatten_pylist(&sublist, flat_list)?;
            }
        }
    }
    Ok(())
}

#[derive(FromPyObject)]
pub enum Arg {
    Scalar(f64),
    List(Py<PyList>),
    Any(Py<PyAny>),
}

impl ToPyObject for Arg {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            Arg::Scalar(s) => s.into_py(py),
            Arg::List(s) => s.into_py(py),
            Arg::Any(s) => s.into_py(py),
        }
    }
}

impl TryFrom<Arg> for ndarray::ArrayD<f64> {
    type Error = PyErr;

    fn try_from(value: Arg) -> Result<Self, Self::Error> {
        let a = match value {
            Arg::Scalar(s) => ndarray::arr1(&[s]).into_dyn(),
            Arg::List(l) => Python::with_gil(|py| pylist_to_arrayd(l.as_ref(py)).unwrap()),
            Arg::Any(obj) => {
                Python::with_gil(|py| {
                    if !numpy::get_array_module(py).is_ok() {
                        unimplemented!();
                    }

                    if let Ok(a) = obj.downcast::<numpy::PyArrayDyn<i64>>(py) {
                        return Ok(a.cast::<f64>(false).unwrap().to_owned_array());
                    }

                    if let Ok(a) = obj.downcast::<numpy::PyArrayDyn<f64>>(py) {
                        return Ok(a.to_owned_array());
                    }

                    Err(PyTypeError::new_err(""))
                })?

                // use numpy::npyffi::PY_ARRAY_API;

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
                // })
            }
        };
        Ok(a)
    }
}

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
}
