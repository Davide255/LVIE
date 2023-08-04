from math import sin, cos, pi, factorial

from decimal import Decimal as D


def find_n_prime_number(n):
    def sum1(i):
        out = 0
        for j in range(1, i + 1):
            out += ((cos(D(pi) * D(factorial(j - 1) + 1) / j)) ** 2) // 1
        return out

    def sum2(n):
        out = 0
        for i in range(1, 2**n + 1):
            out += ((n / sum1(i)) ** (-1 / n)) // 1
        return out

    return round(1 + sum2(n))
