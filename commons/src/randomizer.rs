//! Инструменты для генерации случайных данных и последовательностей.

use rand::distr::uniform::SampleUniform;
use rand::prelude::*;

/// Выбрать случайный элемент из массива или вектора строк.
///
/// ## Пример
///
/// ```
/// use commons::randomizer::random_choice_str;
///
/// let seq: [&str; 3] = ["one", "two", "three"];
/// let result = random_choice_str(&seq).unwrap();
///
/// println!("I said: {}", result);
/// ```
///
/// ## Returns
///
/// Случайный элемент из массива как `String`. Если массив пустой, то `None`.
pub fn random_choice_str<I, S>(seq: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut rng = rand::rng();
    seq.into_iter()
        .choose(&mut rng)
        .map(|s| s.as_ref().to_string())
}

/// Генерировать случайное число из заданного числового диапазона
/// (включительно `max`).
///
/// ## Пример
///
/// ```
/// use commons::randomizer::random;
///
/// let num = random(10, 25);
/// println!("Lucky num: {}", num);
/// ```
///
/// ## Returns
///
/// Случайное число того же типа, что предоставленные для диапазона.
pub fn random<T>(min: T, max: T) -> T
where
    T: SampleUniform + PartialOrd,
{
    let mut rng = rand::rng();
    rng.random_range(min..=max)
}
