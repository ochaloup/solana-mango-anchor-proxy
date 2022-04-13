# python path hacking, it's fine for test purposes
import sys
import os.path
import logging
import asyncio
import json

from pathlib import Path
import typing
from solana.rpc.async_api import AsyncClient
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.transaction import TransactionSignature, AccountMeta
from anchorpy import Idl, Program, Context, Provider, Wallet
from mangohelper import get_symbol
from mango import Account, AccountInfo, Cache, ContextBuilder, ClusterUrlData, Group
from decimal import Decimal
from mango.porcelain import market as load_loadedmarket, LoadedMarket, PerpMarket
from mango import PerpMarketDetails
from mango.constants import I64_MAX

# # Sysprogram needed for creating the account
# from solana.system_program import SYS_PROGRAM_ID
# # Used when multi instructions in one txn is run
# from solana.transaction import Transaction

parent_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
sys.path.insert(0, str(Path(parent_dir).joinpath("tests")))
import marketcontract  # noqa: E402


logging.basicConfig(level=logging.INFO)

SEED = b"market-contract"
TEST_MARKET_INDEX = 42

MANGO_GROUP_NAME = "devnet"
RPC_NODE = "https://mango.devnet.rpcpool.com"
MANGO_MARKET_SYMBOL = "SOL-PERP"

# Devnet program and group
# mango_program = PublicKey("4skJ85cdxQAFVKbcGgfun8iZPL7BadVYXG3kGEGkufqA")
# mango_group = PublicKey("Ec2enZyoC4nGpEfu2sUNAa2nUGJHWxoUWYSEJ2hNTWTA")

# Address of the deployed program
program_id = PublicKey("B8hvuv3LXchAe4Wm5EVAKUntUsymGyAh1n8dfM5KuR3d")
# Address of the Mango Account of the user
user_mango_account_pk = PublicKey("CNg4H4VTdyMWysHG3QhepqXYTZima1g1wSwfzaLHRFmh")


# these calculations are taken from Mango Explorer
def get_perp_quantities(
    price: Decimal,
    quantity: Decimal,
    perp_market_details: PerpMarketDetails,
    max_quote_quantity: Decimal = Decimal(0),
) -> typing.Tuple[Decimal, Decimal, Decimal]:
    base_decimals = perp_market_details.base_instrument.decimals
    quote_decimals = perp_market_details.quote_token.token.decimals

    base_factor = Decimal(10) ** base_decimals
    quote_factor = Decimal(10) ** quote_decimals

    native_price = ((price * quote_factor) * perp_market_details.base_lot_size) / (
        perp_market_details.quote_lot_size * base_factor
    )
    native_quantity = (quantity * base_factor) / perp_market_details.base_lot_size
    native_max_quote_quantity = (
        (max_quote_quantity * quote_factor) / perp_market_details.quote_lot_size
    ) or I64_MAX
    return (native_price, native_quantity, native_max_quote_quantity)


async def main():
    # Read the Anchor IDL
    with Path(parent_dir).joinpath("target/idl/market_contract.json").open() as f:
        raw_idl = json.load(f)
    idl = Idl.from_json(raw_idl)

    # Read Solana Wallet
    path = Path(Path.home() / ".config/solana/id.json")
    with path.open() as f:
        keypair = json.load(f)
    mykeypair = Keypair.from_secret_key(bytes(keypair))

    # Read Mango config
    mango_symbol_json = get_symbol(MANGO_GROUP_NAME, MANGO_MARKET_SYMBOL)
    print(f'mango group pubkey:\n{mango_symbol_json}')

    mango_context: Context = ContextBuilder.build(
        name=MANGO_GROUP_NAME,
        cluster_name=mango_symbol_json["cluster"],
        group_name=mango_symbol_json["name"],
        group_address=mango_symbol_json["publicKey"],
        cluster_urls=[ClusterUrlData(rpc=RPC_NODE)],
        program_address=mango_symbol_json['mangoProgramId'],
        serum_program_address=mango_symbol_json['serumProgramId'],
    )
    user_mango_account: AccountInfo = AccountInfo.load(mango_context, user_mango_account_pk)
    mango_group_pk: PublicKey = PublicKey(mango_symbol_json["publicKey"])
    mango_group: Group = Group.load(mango_context, mango_group_pk)
    mango_cache: Cache = Cache.load(mango_context, mango_group.cache)
    account: Account = Account.parse(user_mango_account, mango_group, mango_cache)
    loaded_market: LoadedMarket = load_loadedmarket(mango_context, MANGO_MARKET_SYMBOL)
    perp_market: PerpMarket = PerpMarket.ensure(loaded_market)

    # For use on local network
    # anchorpy_provider = Provider.local()
    #
    async_client = AsyncClient(endpoint=RPC_NODE)
    anchorpy_provider = Provider(async_client, Wallet.local())

    # Context manager closes the http client (or program.close() is needed)
    async with Program(idl, program_id, anchorpy_provider) as program:
        # Execute the RPC.
        logging.info(f"Running program {program_id}")

        (pda_market_account, bump) = marketcontract.get_pda_key(program, TEST_MARKET_INDEX)
        print(
            f'PDA: {pda_market_account}, wallet pubkey: {program.provider.wallet.public_key},'
            f' keypair pubkey: {mykeypair.public_key}'
        )

        # # Multi instruction call
        # ix1 = program.instruction["cancel_all"](
        #     ctx=Context(
        #         accounts={
        #             "authority": program.provider.wallet.public_key,
        #             "pda_market_account": pda_market_account,
        #             "mango_program": mango_symbol_json['mangoProgramId'],
        #             "mango_group": mango_group_pk,
        #             "mango_account": user_mango_account.address,
        #             "perp_market": mango_symbol_json['market']['publicKey'],
        #             "mango_bids": mango_symbol_json['market']['bidsKey'],
        #             "mango_asks": mango_symbol_json['market']['asksKey'],
        #         },
        #         signers=[program.provider.wallet.payer],
        #     ),
        # )
        # ix2 = program.instruction["cancel_perp_order"](
        #     1649792828776,
        #     True,
        #     ctx=Context(
        #         accounts={
        #             "authority": program.provider.wallet.public_key,
        #             "pda_market_account": pda_market_account,
        #             "mango_program": mango_symbol_json['mangoProgramId'],
        #             "mango_group": mango_group_pk,
        #             "mango_account": user_mango_account.address,
        #             "perp_market": mango_symbol_json['market']['publicKey'],
        #             "mango_bids": mango_symbol_json['market']['bidsKey'],
        #             "mango_asks": mango_symbol_json['market']['asksKey'],
        #         },
        #         signers=[program.provider.wallet.payer],
        #     ),
        # )
        # txn = Transaction()
        # txn.add(ix1)
        # txn.add(ix2)
        # txn.add(ix2)
        # txn.add(ix2)
        # signature = await anchorpy_provider.send(txn)
        # logging.info(f"Program {program_id} was run sucesfully, outcoming txn: {signature}")

        # # Creating the PDA account
        # txn: TransactionSignature = await program.rpc["create"](
        #     TEST_MARKET_INDEX,
        #     ctx=Context(
        #         accounts={
        #             "pda_market_account": pda_market_account,
        #             "authority": program.provider.wallet.public_key,
        #             "system_program": SYS_PROGRAM_ID,
        #         },
        #         signers=[],
        #     ),
        # )
        # logging.info(f"Program {program_id} was run sucesfully, outcoming txn: {txn}")
        # return

        # Placing perp order
        open_orders_pks = [
            AccountMeta(pubkey=open_order, is_signer=False, is_writable=False)
            for open_order in account.spot_open_orders
        ]
        accounts = {
            "authority": program.provider.wallet.public_key,
            "pda_market_account": pda_market_account,
            "mango_account": user_mango_account.address,
            "mango_program": mango_symbol_json['mangoProgramId'],
            "mango_group": mango_group_pk,
            "perp_market": mango_symbol_json['market']['publicKey'],
            "mango_cache": mango_group.cache,
            "mango_bids": mango_symbol_json['market']['bidsKey'],
            "mango_asks": mango_symbol_json['market']['asksKey'],
            "mango_event_queue": mango_symbol_json['market']['eventsKey'],
        }
        print(
            f'Running with:\n{accounts}\n'
            f'remaining accounts:{open_orders_pks}\nsigners:{[mykeypair]}'
        )
        (native_price, native_quantity, native_max_quote_quantity) = get_perp_quantities(
            Decimal(100), Decimal(0.5), perp_market.underlying_perp_market
        )
        # print(f'>>>> {(native_price, native_quantity, native_max_quote_quantity) }')
        txn: TransactionSignature = await program.rpc["place_order"](
            42,  # client id
            0,  # 0 BUY, 1 SELL
            int(native_price),  # price
            int(native_quantity),  # max_base_quantity
            int(native_max_quote_quantity),  # max_quote_quantity
            ctx=Context(
                accounts=accounts,
                remaining_accounts=open_orders_pks,
                signers=[program.provider.wallet.payer],
            ),
        )
        logging.info(f"Program {program_id} was run sucesfully, outcoming txn: {txn}")

        # # Cancel all
        # txn: TransactionSignature = await program.rpc["cancel_all"](
        #     ctx=Context(
        #         accounts={
        #             "authority": program.provider.wallet.public_key,
        #             "pda_market_account": pda_market_account,
        #             "mango_program": mango_symbol_json['mangoProgramId'],
        #             "mango_group": mango_group_pk,
        #             "mango_account": user_mango_account.address,
        #             "perp_market": mango_symbol_json['market']['publicKey'],
        #             "mango_bids": mango_symbol_json['market']['bidsKey'],
        #             "mango_asks": mango_symbol_json['market']['asksKey'],
        #         },
        #         signers=[program.provider.wallet.payer],
        #     ),
        # )
        # logging.info(f"Program {program_id} was run sucesfully, outcoming txn: {txn}")

        # # Cancel perp by order id
        # txn: TransactionSignature = await program.rpc["cancel_perp_order"](
        #     1649792828776,
        #     False,
        #     ctx=Context(
        #         accounts={
        #             "authority": program.provider.wallet.public_key,
        #             "pda_market_`account": pda_market_account,
        #             "mango_program": mango_symbol_json['mangoProgramId'],
        #             "mango_group": mango_group_pk,
        #             "mango_account": user_mango_account.address,
        #             "perp_market": mango_symbol_json['market']['publicKey'],
        #             "mango_bids": mango_symbol_json['market']['bidsKey'],
        #             "mango_asks": mango_symbol_json['market']['asksKey'],
        #         },
        #         signers=[program.provider.wallet.payer],
        #     ),
        # )
        # logging.info(f"Program {program_id} was run sucesfully, outcoming txn: {txn}")

        # # Place perp proxy order
        # open_orders_pks = [
        #     AccountMeta(pubkey=open_order, is_signer=False, is_writable=False)
        #     for open_order in account.spot_open_orders
        # ]
        # accounts = {
        #     "authority": program.provider.wallet.public_key,
        #     "pda_market_account": pda_market_account,
        #     "mango_account": user_mango_account.address,
        #     "mango_program": mango_symbol_json['mangoProgramId'],
        #     "mango_group": mango_group_pk,
        #     "perp_market": mango_symbol_json['market']['publicKey'],
        #     "mango_cache": mango_group.cache,
        #     "mango_bids": mango_symbol_json['market']['bidsKey'],
        #     "mango_asks": mango_symbol_json['market']['asksKey'],
        #     "mango_event_queue": mango_symbol_json['market']['eventsKey'],
        # }
        # print(
        #     f'Running with:\n{accounts}\n'
        #     f'remaining accounts:{open_orders_pks}\nsigners:{[mykeypair]}'
        # )
        # txn: TransactionSignature = await program.rpc["place_order_proxy"](
        #     42,
        #     0,  # 0 BUY, 1 SELL
        #     100,
        #     22,
        #     33,
        #     ctx=Context(
        #         accounts=accounts,
        #         remaining_accounts=open_orders_pks,
        #         signers=[program.provider.wallet.payer],
        #     ),
        # )
        # logging.info(f"Program {program_id} was run sucesfully, outcoming txn: {txn}")


asyncio.run(main())
