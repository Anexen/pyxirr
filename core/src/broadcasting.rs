use std::{error::Error, fmt};

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
