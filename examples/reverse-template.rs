use std::io;

fn main() -> io::Result<()> {
    let line = io::stdin().lines().next().unwrap().unwrap();
    let rev = Solution::reverse(&line);
    println!("{}", rev);
    Ok(())
}

struct Solution;
trait SolutionTrait {
    fn reverse(line: &str) -> String;
}

mod solution {
    impl crate::SolutionTrait for crate::Solution {
        // BASALT_SOLUTION_START
        fn reverse(line: &str) -> String {
            line.chars().rev().collect()
        }
        // BASALT_SOLUTION_END

        // BASALT_TEMPLATE_START
        // fn reverse(line: &str) -> String {
        //     // Your solution here
        // }
        // BASALT_TEMPLATE_END
    }
}
