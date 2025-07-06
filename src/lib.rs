pub mod currency;
pub mod game;
pub mod player;
pub mod world;

pub type Result<T> = color_eyre::Result<T>;

/// Try to downcast any array of [u8] into an array of constant size
pub fn len_to_const_arr<'a, const N: usize, T: 'a>(data: &'a [T]) -> Result<[T; N]>
where
    [T; N]: std::convert::TryFrom<&'a [T]>,
{
    let arr: [T; N] = match data.try_into() {
        Ok(v) => v,
        Err(_e) => {
            // TODO: use a proper error type
            panic!("! Data is of bad length {}", data.len());
        }
    };
    Ok(arr)
}
