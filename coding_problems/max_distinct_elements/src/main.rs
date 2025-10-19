// Maximum Number of Distinct Elements After Operations
struct Solution;
impl Solution {
    pub fn max_distinct_elements(mut nums: Vec<i32>, k: i32) -> i32 {
        nums.sort();
        let mut prev = i32::MIN;
        let mut cnt = 0;
        for n in nums {
            let v = (n+k).min((n-k).max(prev+1));
            if v > prev {
                cnt += 1;
                prev = v;
            }
        }
        cnt
    }
}

fn main() {
    let nums = vec![2,4,3,2,5,5,5];
    println!("{}", Solution::max_distinct_elements(nums, 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let nums = vec![2,4,3,2,5,5,5];
        assert_eq!(Solution::max_distinct_elements(nums, 3), 7);
    }

    #[test]
    fn test_2() {
        let nums = vec![1,2,3,4,5];
        assert_eq!(Solution::max_distinct_elements(nums, 1), 5);
    }

    #[test]
    fn test_3() {
        let nums = vec![5,5,5,5,5];
        assert_eq!(Solution::max_distinct_elements(nums, 1), 3);
    }
}