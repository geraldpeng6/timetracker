// 函数式编程工具
// 提供常用的函数式编程模式和工具

use std::collections::HashMap;
use std::hash::Hash;

/// 管道操作符 - 允许链式调用
pub trait Pipe<T> {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(T) -> R;
}

impl<T> Pipe<T> for T {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        f(self)
    }
}

/// 条件应用 - 只在条件为真时应用函数
pub trait ConditionalApply<T> {
    fn apply_if<F>(self, condition: bool, f: F) -> T
    where
        F: FnOnce(T) -> T;

    fn apply_if_some<F, U>(self, option: Option<U>, f: F) -> T
    where
        F: FnOnce(T, U) -> T;
}

impl<T> ConditionalApply<T> for T {
    fn apply_if<F>(self, condition: bool, f: F) -> T
    where
        F: FnOnce(T) -> T,
    {
        if condition {
            f(self)
        } else {
            self
        }
    }

    fn apply_if_some<F, U>(self, option: Option<U>, f: F) -> T
    where
        F: FnOnce(T, U) -> T,
    {
        match option {
            Some(value) => f(self, value),
            None => self,
        }
    }
}

/// 分组函数 - 将集合按键分组
pub fn group_by<T, K, F>(items: Vec<T>, key_fn: F) -> HashMap<K, Vec<T>>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    items.into_iter().fold(HashMap::new(), |mut acc, item| {
        let key = key_fn(&item);
        acc.entry(key).or_default().push(item);
        acc
    })
}

/// 聚合函数 - 对分组后的数据进行聚合
pub fn aggregate<T, K, V, F, G>(items: Vec<T>, key_fn: F, agg_fn: G) -> HashMap<K, V>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
    G: Fn(Vec<T>) -> V,
{
    group_by(items, key_fn)
        .into_iter()
        .map(|(k, v)| (k, agg_fn(v)))
        .collect()
}

/// 过滤映射 - 同时过滤和映射
pub fn filter_map<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
where
    F: Fn(T) -> Option<U>,
{
    items.into_iter().filter_map(f).collect()
}

/// 分区函数 - 将集合分为两部分
pub fn partition<T, F>(items: Vec<T>, predicate: F) -> (Vec<T>, Vec<T>)
where
    F: Fn(&T) -> bool,
{
    items.into_iter().partition(predicate)
}

/// 累积映射 - 带状态的映射
pub fn scan<T, U, S, F>(items: Vec<T>, initial_state: S, f: F) -> Vec<U>
where
    F: Fn(S, T) -> (S, U),
    S: Clone,
{
    let mut state = initial_state;
    items
        .into_iter()
        .map(|item| {
            let (new_state, result) = f(state.clone(), item);
            state = new_state;
            result
        })
        .collect()
}

/// 窗口函数 - 滑动窗口处理
pub fn windowed<T>(items: Vec<T>, size: usize) -> Vec<Vec<T>>
where
    T: Clone,
{
    if size == 0 || items.len() < size {
        return vec![];
    }

    (0..=items.len() - size)
        .map(|i| items[i..i + size].to_vec())
        .collect()
}

/// 去重函数 - 保持顺序的去重
pub fn unique<T>(items: Vec<T>) -> Vec<T>
where
    T: Eq + Hash + Clone,
{
    let mut seen = std::collections::HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

/// 频率统计
pub fn frequency<T>(items: Vec<T>) -> HashMap<T, usize>
where
    T: Eq + Hash,
{
    items.into_iter().fold(HashMap::new(), |mut acc, item| {
        *acc.entry(item).or_insert(0) += 1;
        acc
    })
}

/// 组合器 - 组合多个函数
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

/// 柯里化 - 将多参数函数转换为单参数函数链
pub fn curry2<A, B, C, F>(f: F) -> impl Fn(A) -> Box<dyn Fn(B) -> C>
where
    F: Fn(A, B) -> C + Clone + 'static,
    A: Clone + 'static,
    B: 'static,
    C: 'static,
{
    move |a| {
        let f_clone = f.clone();
        let a_clone = a.clone();
        Box::new(move |b| f_clone(a_clone.clone(), b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe() {
        let result = 5.pipe(|x| x * 2).pipe(|x| x + 1);
        assert_eq!(result, 11);
    }

    #[test]
    fn test_group_by() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let grouped = group_by(items, |x| x % 2);

        assert_eq!(grouped.get(&0), Some(&vec![2, 4, 6]));
        assert_eq!(grouped.get(&1), Some(&vec![1, 3, 5]));
    }

    #[test]
    fn test_frequency() {
        let items = vec!["a", "b", "a", "c", "b", "a"];
        let freq = frequency(items);

        assert_eq!(freq.get("a"), Some(&3));
        assert_eq!(freq.get("b"), Some(&2));
        assert_eq!(freq.get("c"), Some(&1));
    }
}
