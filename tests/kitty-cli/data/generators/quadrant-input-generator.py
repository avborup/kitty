import random


def pick_random_nonzero_int():
    x = 0
    while x == 0:
        x = random.randint(-1000, 1000)
    return x


print(pick_random_nonzero_int())
print(pick_random_nonzero_int())
