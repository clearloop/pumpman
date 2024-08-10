#!/usr/bin/env python3

# k - marketcap
# x - total reserve
# y - current supply
#
# k = (x0 + xn) * (y0 - yn)


# know cases
#
# 1. 28 = 10^9
# y = 10^9 - 793100000
# x = 0
# k = 28

SUPPLY_GT_85 = 793100000
print()
def mc(sol):
    # k = (30 + x) * (1073000000 - y)
    y = 1073000000 - 32190000000 / (30 + float(sol));
    print("total supply (k):", y);

sol = input("total reserve (y): ")
mc(sol)
