class Solution:
    @staticmethod
    # BASALT_SOLUTION_START
    def reverse(line: str) -> str:
        return ''.join(reversed(list(line)))
    # BASALT_SOLUTION_END

    # BASALT_TEMPLATE_START
    # def reverse(line: str) -> str:
    #     # Your solution here
    # BASALT_TEMPLATE_END

print(Solution.reverse(input()))
