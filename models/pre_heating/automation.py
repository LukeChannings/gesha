import asyncio
from datetime import datetime
from api import Gesha

WAIT_TIME = 60 * 60


async def automation():
    gesha = Gesha(subscribe_topics=[])

    # Move into manual control mode.
    gesha.set_control_method("pid")
    gesha.set_mode("active")

    print("EXPERIMENT BEGINS")

    for target_temp in range(93, 101):
        print(
            f"Setting the target temperature to {target_temp} at {datetime.now().strftime('%H:%M:%S')}"
        )
        gesha.set_target_temperature(target_temp)
        await asyncio.sleep(WAIT_TIME)

    gesha.set_mode("idle")


def main():
    asyncio.run(automation())


if __name__ == "__main__":
    main()
