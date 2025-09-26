use std::collections::HashMap;

struct Solution;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let mut map: HashMap::<i32,i32> = HashMap::new();
        for i in 0..nums.len() {
            let n = target - nums[i];
            if let Some(&k) = map.get(&n) {
                return vec![k, i as i32];
            }
            map.insert(nums[i], i as i32);
        }
        return vec![];
    }
}

fn main() {
    let nums = vec![2, 7, 11, 15];
    let target = 9;
    let result = Solution::two_sum(nums, target);
    println!("{:?}", result);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test1() {
        let nums = vec![2, 7, 11, 15];
        let target = 9;
        let result = Solution::two_sum(nums, target);
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test2() {
        let nums = vec![3, 2, 4];
        let target = 6;
        let result = Solution::two_sum(nums, target);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test3() {
        let nums = vec![3, 3];
        let target = 6;
        let result = Solution::two_sum(nums, target);
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn no_solution() {
        let nums = vec![1, 2, 3];
        let target = 7;
        let result = Solution::two_sum(nums, target);
        assert_eq!(result, vec![]);
    }
}
