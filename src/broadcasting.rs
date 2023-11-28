use std::{error::Error, fmt};

use ndarray::{ArrayD, ArrayViewD, Axis, CowArray, IxDyn};
use numpy::{npyffi, Element, PyArrayDyn, PY_ARRAY_API};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyIterator, PyList, PySequence, PyTuple},
};

use crate::conversions::float_or_none;

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
    for (i, cur) in result.iter_mut().enumerate() {
        *cur = 1;
        for s in shapes.iter() {
            /* This prepends 1 to shapes not already equal to ndim */
            if i + s.len() >= ndim {
                let k = i + s.len() - ndim;
                let tmp = s[k];
                if tmp == 1 {
                    continue;
                }
                if cur == &1 {
                    *cur = tmp;
                } else if cur != &tmp {
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

            match $crate::broadcasting::broadcast_shapes(_a) {
                Some(shape) => Ok(( $($a.broadcast(shape.clone()).unwrap(),)*)),
                None => Err(BroadcastingError::new(_a))
            }
        }
    };
}

pub fn pyiter_to_arrayd<'p, T>(pyiter: &'p PyIterator) -> PyResult<ArrayD<T>>
where
    T: FromPyObject<'p>,
{
    let mut dims = Vec::new();
    let mut flat_list = Vec::new();
    flatten_pyiter(pyiter, &mut dims, &mut flat_list, 0)?;
    let arr = ArrayD::from_shape_vec(IxDyn(&dims), flat_list);
    arr.map_err(|e| PyValueError::new_err(e.to_string()))
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

fn flatten_pyiter<'p, T>(
    pyiter: &'p PyIterator,
    shape: &mut Vec<usize>,
    flat_list: &mut Vec<T>,
    depth: usize,
) -> PyResult<()>
where
    T: FromPyObject<'p>,
{
    let mut max_i = 0;
    for (i, item) in pyiter.enumerate() {
        let item = item?;
        max_i = i;
        match item.extract::<T>() {
            Ok(val) => flat_list.push(val),
            Err(_) => {
                let sublist = item.iter()?;
                flatten_pyiter(sublist, shape, flat_list, depth + 1)?;
            }
        }
    }

    max_i += 1;
    if let Some(current) = shape.get(depth) {
        shape[depth] = (*current).max(max_i);
    } else {
        shape.push(max_i);
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
    pub fn into_arrayd(self) -> CowArray<'p, T, IxDyn> {
        self.into()
    }
}

fn is_numpy_available() -> bool {
    Python::with_gil(|py| py.import("numpy").is_ok())
}

fn pyarray_cast<U: Element>(ob: &PyAny) -> PyResult<&PyArrayDyn<U>> {
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

        if ob.downcast::<PyList>().is_ok()
            || ob.downcast::<PyTuple>().is_ok()
            || ob.downcast::<PyIterator>().is_ok()
            || ob.downcast::<PySequence>().is_ok()
        {
            let arr = pyiter_to_arrayd(ob.iter()?)?;
            return Ok(Arg::Array(CowArray::from(arr)));
        }

        if is_numpy_available() {
            if let Ok(a) = ob.downcast::<numpy::PyArrayDyn<f64>>() {
                return Ok(Arg::NumpyArray(a));
            }

            if unsafe { npyffi::PyArray_Check(ob.py(), ob.as_ptr()) } == 1 {
                let a = pyarray_cast::<f64>(ob)?;
                return Ok(Arg::NumpyArray(a));
            }
        }

        Err(PyTypeError::new_err("must be float scalar or array-like"))
    }
}

impl<'p> FromPyObject<'p> for Arg<'p, bool> {
    fn extract(ob: &'p PyAny) -> PyResult<Self> {
        if let Ok(value) = ob.extract::<bool>() {
            return Ok(Arg::Scalar(value));
        };

        if ob.downcast::<PyList>().is_ok()
            || ob.downcast::<PyTuple>().is_ok()
            || ob.downcast::<PyIterator>().is_ok()
            || ob.downcast::<PySequence>().is_ok()
        {
            let arr = pyiter_to_arrayd(ob.iter()?)?;
            return Ok(Arg::Array(CowArray::from(arr)));
        }

        if is_numpy_available() {
            if let Ok(a) = ob.downcast::<numpy::PyArrayDyn<bool>>() {
                return Ok(Arg::NumpyArray(a));
            }
        }

        Err(PyTypeError::new_err("must be bool scalar or array-like"))
    }
}

impl IntoPy<PyObject> for Arg<'_, f64> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

impl ToPyObject for Arg<'_, f64> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
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

impl<'p, T> From<&'p PyArrayDyn<T>> for Arg<'p, T> {
    fn from(arr: &'p PyArrayDyn<T>) -> Self {
        Arg::NumpyArray(arr)
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

    #[rstest]
    fn test_flatten_pyiter() {
        Python::with_gil(|py| {
            let ob = py.eval("(range(i, i + 3) for i in range(3))", None, None).unwrap();
            let array = pyiter_to_arrayd::<i64>(ob.iter().unwrap()).unwrap();
            let expected = ndarray::array![[0, 1, 2], [1, 2, 3], [2, 3, 4]].into_dyn();
            assert!(array == expected)
        });
    }
}
