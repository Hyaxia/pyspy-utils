import time


def another_func():
    start = time.time()
    while time.time() - start < 3:  # run for 3 seconds
        i = 100000 * 400000


def my_func():
    print("running now cpu intensive task")
    another_func()
    print("done")
    time.sleep(1)


if __name__ == "__main__":
    while True:
        my_func()
