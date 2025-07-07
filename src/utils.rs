use crate::{Result, err_int};

/// Try to downcast any array of [u8] into an array of constant size
pub(crate) fn len_to_const_arr<'a, const N: usize, T: 'a>(data: &'a [T]) -> Result<[T; N]>
where
    [T; N]: std::convert::TryFrom<&'a [T]>,
{
    let arr: [T; N] = match data.try_into() {
        Ok(v) => v,
        Err(_) => {
            return Err(err_int!(
                "Data length mismatch: expected {}, got {}",
                N,
                data.len()
            ));
        }
    };
    Ok(arr)
}
