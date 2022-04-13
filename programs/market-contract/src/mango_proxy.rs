use anchor_lang::prelude::*;
use solana_program::entrypoint::ProgramResult;
use mango::matching::OrderType;
use mango::matching::Side;
use mango::state::MAX_PAIRS;

mod mango_program_id {
    #[cfg(feature = "development")]
    solana_program::declare_id!("4skJ85cdxQAFVKbcGgfun8iZPL7BadVYXG3kGEGkufqA");
    #[cfg(feature = "production")]
    solana_program::declare_id!("mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68");
}

#[derive(Clone)]
pub struct MangoMarketV3;

impl anchor_lang::Id for MangoMarketV3 {
    fn id() -> Pubkey {
        mango_program_id::ID
    }
}

pub fn place_perp_order2<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, PlacePerpOrder2<'info>>,
    side: Side,
    price: i64,
    max_base_quantity: i64,
    max_quote_quantity: i64,
    client_order_id: u64,
    order_type: OrderType,
    reduce_only: bool,
    expiry_timestamp: Option<u64>,  // 0 to ignore time forcing
    limit: u8,  // max number of fill events before terminating (use u8 max for "no limit")
) -> Result<()> {
    let remaining_accounts_iter = ctx.remaining_accounts.iter();
    // let referral = remaining_accounts_iter.next();
    // let mut open_orders = vec![Pubkey::default(); MAX_PAIRS];
    // remaining_accounts_iter.for_each(|ai| open_orders.push(*ai.key));
    let open_orders = [];
    let ix = mango::instruction::place_perp_order2(
        &mango_program_id::ID,
        ctx.accounts.mango_group.key,
        ctx.accounts.mango_account.key,
        ctx.accounts.owner.key,
        ctx.accounts.mango_cache.key,
        ctx.accounts.perp_market.key,
        ctx.accounts.bids.key,
        ctx.accounts.asks.key,
        ctx.accounts.event_queue.key,
        None,  // referral.map(|r| r.key),
        &open_orders,
        side,
        price,
        max_base_quantity,
        max_quote_quantity,
        client_order_id,
        order_type,
        reduce_only,
        expiry_timestamp,
        limit,
    )?;
    // msg!("iiix: {:?}", ix);
    let result: ProgramResult = solana_program::program::invoke(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
    );
    msg!("Invocation result: {:?}", result);
    result.map_err(Into::into)
}


#[derive(Accounts)]
pub struct PlacePerpOrder2<'info> {
    /// CHECK: Mango CPI
    pub mango_group: AccountInfo<'info>,
    /// CHECK: Mango CPI
    pub mango_account: AccountInfo<'info>,
    /// CHECK: Mango CPI
    pub owner: AccountInfo<'info>,
    /// CHECK: Mango CPI
    pub mango_cache: AccountInfo<'info>,
    /// CHECK: Mango CPI
    pub perp_market: AccountInfo<'info>,
    /// CHECK: Mango CPI
    pub bids: AccountInfo<'info>,
    /// CHECK: Mango CPI
    pub asks: AccountInfo<'info>,
    /// CHECK: Mango CPI
    pub event_queue: AccountInfo<'info>,
    /// CHECK: Mango CPI
    pub referral: AccountInfo<'info>,

    // referencing OpenOrders as list [0-MAX_PAIRS] of the `remaining_accounts` Vec
}
