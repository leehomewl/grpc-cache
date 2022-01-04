from typing import Dict
from time import time_ns

import asyncio

from concurrent.futures import ThreadPoolExecutor

READ_THROTTLE = 0.000001
WRITE_THROTTLE = 0.000001
READ_TIMEOUT = 10
READERS = 4
READ_BATCH = 1_000_000;
READ_ITERS = 100_000_000;
WRITE_BATCH = 1000;
WRITE_ITERS = 10_000_000;

cache: Dict = {}

async def put(key, value):
    cache[key] = value

async def get(key):
    return cache.get(key)

async def write():
    for i in range(WRITE_ITERS):
        await put(i, i*100)
        if i % WRITE_BATCH == 0:
            print(f"Writer {i}")
            await asyncio.sleep(WRITE_THROTTLE)

async def read(reader: int):
    max_lat = 0
    count = 0
    total = 0
    for i in range(READ_ITERS):
        start = time_ns()
        v = await get(i)
        elapsed = 10e-6 * (time_ns() - start)
        max_lat = max(max_lat, elapsed)
        total += elapsed
        count += 1
        if i % READ_BATCH == 0:
            print(f"Reader {reader}: Got {i}:{v} Count: {count} Avg:{total/count}ms Max:{max_lat}ms")
            await asyncio.sleep(READ_THROTTLE)


async def main():
    loop = asyncio.get_event_loop()
    with ThreadPoolExecutor() as p:
        tasks = [
            await loop.run_in_executor(p, write),
            await loop.run_in_executor(p, read, 1),
            await loop.run_in_executor(p, read, 2),
            await loop.run_in_executor(p, read, 3),
            await loop.run_in_executor(p, read, 4),
        ]
        print(tasks)
        await asyncio.sleep(0.1)
        results = await asyncio.gather(*tasks)
        print(results)

if __name__ == "__main__":
    asyncio.run(main())


