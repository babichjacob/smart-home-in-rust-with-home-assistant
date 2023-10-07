from asyncio import run

async def smart_home_in_rust_with_home_assistant_main():    
    from smart_home_in_rust_with_home_assistant import main

    await main()

run(smart_home_in_rust_with_home_assistant_main())
