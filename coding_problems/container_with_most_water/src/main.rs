struct Solution;
impl Solution {
    pub fn max_area(height: Vec<i32>) -> i32 {
        let (mut i, mut j) = (0, height.len()-1);
        let mut mx = 0;
        while i < j {
            mx = mx.max((j-i) as i32 * height[i].min(height[j]));
            if height[i] < height[j] {
                i += 1;
            }
            else {
                j -= 1;
            }
        }
        mx
    }
}

fn main() {
    let height = vec![1,8,6,2,5,4,8,3,7];
    println!("{}", Solution::max_area(height));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let height = vec![1,8,6,2,5,4,8,3,7];
        assert_eq!(49, Solution::max_area(height));
    }

    #[test]
    fn test_2() {
        let height = vec![1,1];
        assert_eq!(1, Solution::max_area(height));
    }

    #[test]
    fn test_3() {
        let height = vec![4,3,2,1,4];
        assert_eq!(16, Solution::max_area(height));
    }
}

