pub fn linear_search<T: PartialEq>(arr: &[T], value: &T) -> Option<usize> {
    let mut i = 0;
    while i < arr.len() {
        if &arr[i] == value {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[test]
fn test_linear_search() {
    let arr = [1, 3, 5, 7, 9, 11, 15];
    assert_eq!(linear_search(&arr, &1), Some(0));
    assert_eq!(linear_search(&arr, &3), Some(1));
    assert_eq!(linear_search(&arr, &5), Some(2));
    assert_eq!(linear_search(&arr, &11), Some(5));
    assert_eq!(linear_search(&arr, &15), Some(6));
    assert_eq!(linear_search(&arr, &0), None);
    assert_eq!(linear_search(&arr, &-1), None);
    assert_eq!(linear_search(&arr, &-3), None);

    let arr1 = [1.5, 77.8, -5.3, 1.3, -9.0, 2.5, 5.3];
    assert_eq!(linear_search(&arr1, &1.5), Some(0));
    assert_eq!(linear_search(&arr1, &-5.3), Some(2));
    assert_eq!(linear_search(&arr1, &2.5), Some(5));
    assert_eq!(linear_search(&arr1, &-9.0), Some(4));
    assert_eq!(linear_search(&arr1, &0.0), None);
    assert_eq!(linear_search(&arr1, &1.4), None);
}
