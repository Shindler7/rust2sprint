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

/// Перемешать случайным образом вектор.
pub fn shuffle_vec<T>(mut vec: Vec<T>) -> Vec<T> {
    let mut rng = rand::rng();
    vec.shuffle(&mut rng);
    vec
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

/// Обёртка для функции [`random`]: позволяет генерировать случайное число
/// из диапазона между двумя числами, заданным в кортеже.
pub fn random_by_tuple<T>(t: (T, T)) -> T
where
    T: SampleUniform + PartialOrd,
{
    random(t.0, t.1)
}

/// Случайное значение `true` или `false`, с учётом предоставленного критерия
/// вероятности.
///
/// ## Args
///
/// - `prob` — вероятность результата быть `true`. Значения от 0 до 1.
///
/// Паникует, если `prob < 0` или `prob > 1`.
pub fn random_bool(prob: f64) -> bool {
    let mut rng = rand::rng();
    rng.random_bool(prob)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_choice_str_with_valid_input() {
        let seq = vec!["one", "two", "three"];
        let result = random_choice_str(&seq);

        assert!(result.is_some());
        let chosen = result.unwrap();
        assert!(seq.contains(&chosen.as_str()));
    }

    #[test]
    fn test_random_choice_str_with_empty_input() {
        let seq: Vec<&str> = vec![];
        let result = random_choice_str(&seq);

        assert!(result.is_none());
    }

    #[test]
    fn test_random_choice_str_with_different_containers() {
        // Тестируем с массивом
        let arr = ["apple", "banana"];
        let result_arr = random_choice_str(arr);
        assert!(result_arr.is_some());

        // Тестируем с вектором строк
        let vec_strings = vec!["cat".to_string(), "dog".to_string()];
        let result_vec = random_choice_str(&vec_strings);
        assert!(result_vec.is_some());
    }

    #[test]
    fn test_shuffle_vec() {
        let original = vec![1, 2, 3, 4, 5];
        let shuffled = shuffle_vec(original.clone());

        // Проверяем, что элементы те же
        assert_eq!(shuffled.len(), 5);
        assert!(shuffled.iter().all(|item| original.contains(item)));

        // Проверяем, что порядок изменился (с вероятностью 1/120 это может быть ложным срабатыванием)
        assert_ne!(shuffled, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_random_with_integer_range() {
        let result = random(1, 10);

        // Проверяем границы
        assert!((1..=10).contains(&result));
    }

    #[test]
    fn test_random_with_float_range() {
        let result = random(0.0, 1.0);

        // Проверяем границы
        assert!((0.0..=1.0).contains(&result));
    }

    #[test]
    fn test_random_with_same_min_max() {
        let result = random(42, 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_random_by_tuple() {
        let range = (5, 15);
        let result = random_by_tuple(range);

        assert!((5..=15).contains(&result));
    }

    #[test]
    fn test_random_bool_extremes() {
        // Всегда false
        assert!(!random_bool(0.0));

        // Всегда true
        assert!(random_bool(1.0));
    }

    #[test]
    #[should_panic]
    fn test_random_bool_invalid_negative() {
        random_bool(-0.5);
    }

    #[test]
    #[should_panic]
    fn test_random_bool_invalid_greater_than_one() {
        random_bool(1.5);
    }
}
