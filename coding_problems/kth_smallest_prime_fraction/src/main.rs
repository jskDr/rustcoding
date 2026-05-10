//! Solution for the "K-th Smallest Prime Fraction" problem.
//!
//! Given a sorted array that starts with `1` and is followed by prime numbers,
//! find the `k`-th smallest fraction `arr[i] / arr[j]` where `i < j`.
//! The implementation uses binary search over possible fraction values and
//! counts how many fractions are less than or equal to each midpoint.

struct Solution {}

impl Solution {
    /// Returns the numerator and denominator of the `k`-th smallest prime fraction.
    ///
    /// # Arguments
    ///
    /// * `arr` - A sorted list containing `1` followed by prime numbers.
    /// * `k` - The 1-based rank of the fraction to find.
    ///
    /// # Returns
    ///
    /// A two-element vector `[numerator, denominator]`.
    pub fn kth_smallest_prime_fraction(arr: Vec<i32>, k: i32) -> Vec<i32> {
        // Counts fractions <= `mid` and stores the largest fraction found under
        // that threshold in `results`.
        fn calc_no_min(mid: f32, arr: &Vec<i32>, results: &mut Vec<i32>) -> usize {
            let mut j = 0;
            let n = arr.len();
            let mut count = 0;
            let mut max_min_val = 0.0;
            for i in 0..n {
                while j < n && arr[i] as f32 / arr[j] as f32 > mid {
                    j += 1;
                }
                if j == n {
                    break;
                }
                count += n - j;
                while arr[i] as f32 / arr[j] as f32 > max_min_val {
                    max_min_val = arr[i] as f32 / arr[j] as f32;
                    results[0] = arr[i];
                    results[1] = arr[j];
                }
            }
            count
        }
        let k = k as usize;
        let (mut low, mut high) = (0.0, 1.0);
        let mut results = vec![0, 0];
        while low < high {
            let mid = (low + high) / 2.0;
            let count = calc_no_min(mid, &arr, &mut results);
            if count == k {
                return results;
            } else {
                if count < k {
                    low = mid;
                } else {
                    high = mid;
                } 
            }
        }
        results
    }
}


fn main() {
    println!("Testing code for kth_smallest_prime_fraction. For more testing, please use: cargo test");
    let arr = vec![1, 2, 3, 5];
    let k = 3;
    println!("We are testing now: arr = {:?}, k = {}", arr, k);
    let result = Solution::kth_smallest_prime_fraction(arr, k);
    println!("Result: {:?}", result);

}


#[cfg(test)]
mod tests {
    use super::Solution;

    /// Example from the problem statement.
    #[test]
    fn test1() {
        let arr = vec![1, 2, 3, 5];
        let k = 3;
        let result = Solution::kth_smallest_prime_fraction(arr, k);
        assert_eq!(result, vec![2, 5]);
    }

    /// Smallest valid input where only one fraction exists.
    #[test]
    fn test_single_fraction() {
        let arr = vec![1, 7];
        let k = 1;
        let result = Solution::kth_smallest_prime_fraction(arr, k);
        assert_eq!(result, vec![1, 7]);
    }
}
