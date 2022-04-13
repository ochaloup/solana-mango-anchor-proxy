import asyncio
import typing

from pytest import fixture, mark
from solana.keypair import Keypair
from solana.publickey import PublicKey
from anchorpy import create_workspace, close_workspace, Context, Program
from solana.system_program import SYS_PROGRAM_ID


SEED = b"market-contract"
TEST_MARKET_INDEX = 42


def int_to_bytes(x: int) -> bytes:
    return x.to_bytes((x.bit_length() + 7) // 8, 'little')


def get_pda_key(program: Program, market_index: int) -> typing.Tuple[PublicKey, int]:
    market_index_bytes = int_to_bytes(market_index)
    return PublicKey.find_program_address(
        [SEED, market_index_bytes, program.provider.wallet.public_key.__bytes__()],
        program.program_id,
    )


@fixture(scope="module")
def event_loop():
    """Create an instance of the default event loop for each test case."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@fixture(scope="module")
async def program() -> Program:
    workspace = create_workspace()
    yield workspace["market_contract"]
    await close_workspace(workspace)


@fixture(scope="module")
async def initialized_pda_account(program: Program) -> Keypair:
    (pda_account_pubkey, _) = get_pda_key(program, TEST_MARKET_INDEX)
    print(f'PDA: {pda_account_pubkey}, wallet pubkey: {program.provider.wallet.public_key}')
    await program.rpc["create"](
        TEST_MARKET_INDEX,
        ctx=Context(
            accounts={
                "pda_market_account": pda_account_pubkey,
                "authority": program.provider.wallet.public_key,
                "system_program": SYS_PROGRAM_ID,
            },
            signers=[],
        ),
    )
    return pda_account_pubkey


@mark.asyncio
async def test_is_initialized(program: Program, initialized_pda_account: Keypair) -> None:
    counter_account = await program.account["MarketContractAccount"].fetch(initialized_pda_account)
    assert counter_account.authority == program.provider.wallet.public_key
    assert counter_account.counter == 0


@mark.asyncio
async def test_place_order(program: Program, initialized_pda_account: Keypair) -> None:
    fake_keypair = Keypair()
    await program.rpc["place_order"](
        42,  # client id
        0,  # 0 BUY, 1 SELL
        100,  # price
        22,  # max_base_quantity
        33,  # max_quote_quantity
        ctx=Context(
            accounts={
                "authority": program.provider.wallet.public_key,
                "pda_market_account": initialized_pda_account,
                "mango_account": fake_keypair.public_key,
                "mango_program": fake_keypair.public_key,
                "mango_group": fake_keypair.public_key,
                "perp_market": fake_keypair.public_key,
                "mango_cache": fake_keypair.public_key,
                "mango_bids": fake_keypair.public_key,
                "mango_asks": fake_keypair.public_key,
                "mango_event_queue": fake_keypair.public_key,
            },
            signers=[program.provider.wallet.payer],
        ),
    )
