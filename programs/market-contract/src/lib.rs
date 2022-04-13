mod mango_proxy;

use anchor_lang::prelude::*;
use solana_program::system_program;
use mango::state::MangoAccount;
use mango::matching::{OrderType, Side};
use mango::state::MAX_PAIRS;

use mango_proxy::{place_perp_order2, PlacePerpOrder2, MangoMarketV3};


declare_id!("B8hvuv3LXchAe4Wm5EVAKUntUsymGyAh1n8dfM5KuR3d");

const SEED_PHRASE: &[u8; 15] = b"market-contract";



#[program]
mod market_contract {
    use super::*;

    pub fn create(ctx: Context<Create>, market_index: u8) -> Result<()> {
        let contract_account = &mut ctx.accounts.pda_market_account;
        contract_account.market_index = market_index;
        contract_account.counter = 0;
        contract_account.authority = ctx.accounts.authority.key();
        contract_account.bump = *ctx.bumps.get("pda_market_account").unwrap();
        msg!("PDA market contract account created: {:?}", contract_account.key());
        Ok(())
    }

    // Approach inspired at https://github.com/Is0tope/mango_risk_check/
    pub fn place_order<'info>(
        ctx: Context<'_, '_, '_, 'info, PlaceOrder<'info>>,
        client_order_id: u64,
        side: u8,
        price: i64,
        max_base_quantity: i64,
        max_quote_quantity: i64,
    ) -> Result<()> {
        // let contract_account = &mut ctx.accounts.pda_market_account;
        // msg!("place_order:: authority={}, counter={}", contract_account.authority, contract_account.counter);
        // let mango_account = MangoAccount::load_checked(
        //     &ctx.accounts.mango_account,
        //     &MangoMarketV3::id(),
        //     ctx.accounts.mango_group.key
        // ).unwrap();
        // msg!("Found mango account owner {:?}", mango_account.owner);

        let mango_side = if side == 0 {
            Side::Bid  // BUY
        } else {
            Side::Ask  // SELL
        };
        // msg!("Side: {:?}", mango_side);
        let res = place_perp_order(
            &ctx,
            client_order_id,
            mango_side,
            price,
            max_base_quantity,
            max_quote_quantity
        );
        msg!("Result of placing perp order is {:?}", res);

        Ok(())
    }

    // Approach inspired at https://github.com/UXDProtocol/anchor-comp
    pub fn place_order_proxy<'info>(
        ctx: Context<'_, '_, '_, 'info, PlaceOrder<'info>>,
        client_order_id: u64,
        side: u8,
        price: i64,
        max_base_quantity: i64,
        max_quote_quantity: i64,
    ) -> Result<()> {
        let mango_side = if side == 0 {
            Side::Bid  // BUY
        } else {
            Side::Ask  // SELL
        };
        let res = place_perp_order_proxy(
            &ctx,
            client_order_id,
            mango_side,
            price,
            max_base_quantity,
            max_quote_quantity
        );
        msg!("Result of placing perp order is {:?}", res);

        Ok(())
    }

    pub fn cancel_all<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelAll<'info>>,
) -> Result<()> {
        let contract_account = &mut ctx.accounts.pda_market_account;
        msg!("PDA contract, authority={}, counter={}", contract_account.authority, contract_account.counter);

        let res = cancel_all_perp_orders(&ctx);
        msg!("Result of cancelling all perp orders is {:?}", res);

        Ok(())
    }

    pub fn cancel_perp_order<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelPerpOrderByClientId<'info>>,
        client_order_id: u64,
        invalid_id_ok: bool
) -> Result<()> {
        let contract_account = &mut ctx.accounts.pda_market_account;
        msg!("PDA contract, authority={}, counter={}", contract_account.authority, contract_account.counter);

        let res = cancel_perp_order_by_client_id(
            &ctx,  client_order_id, invalid_id_ok
        );
        msg!("Result of cancelling all perp orders is {:?}", res);

        Ok(())
    }
}

pub fn place_perp_order<'info>(
    ctx: &Context<'_, '_, '_, 'info, PlaceOrder<'info>>,
    client_order_id: u64,
    side: Side,
    price: i64,
    max_base_quantity: i64,
    max_quote_quantity: i64,
) {
    // msg!("[--1--]");
    // let mut remaining_accounts_iter = ctx.remaining_accounts.iter();
    // msg!("[--2--]");
    // let mut open_orders = vec![];
    // msg!("[--3--] {:?}", open_orders);
    // let a = remaining_accounts_iter.next();
    // msg!("[---4---] {:?}", a);
    // remaining_accounts_iter.for_each(|ai| {
    //     msg!("Placing key: {}", *ai.key);
    //     open_orders.push(*ai.key);
    // });
    // let mut open_orders: Vec<Pubkey> = vec![Pubkey::default(); MAX_PAIRS];
    // let mut remaining_accounts_iter = ctx.remaining_accounts.iter();
    // let mut i: usize = 0;
    // while let Some(account) = remaining_accounts_iter.next() {
    //     open_orders[i] = *account.key;
    //     i = i + 1;
    // }
    // msg!("Open orders: {:?}, len: {}", open_orders, open_orders.len());
    // for i in 0..15 {
    //     msg!("Iteration {}", i);
    // }
    // for i in 0..open_orders.len() {
    //     msg!("Iteration {}", i);
    // }
    // for i in 0..open_orders.len() {
    //     msg!("Iteration {}", i);
    //     if let Some(account) = remaining_accounts_iter.next() {
    //         msg!("Open orders key definition of {}", i);
    //         msg!(">>>> {}", *account.key);
    //         open_orders[i] = *account.key;
    //     } else {
    //         break
    //     }
    // }

    // msg!("Going to create instruction while open orders keys is: {:?}", open_orders);
    // msg!(">>>: {:?}", open_orders.as_slice());
    // msg!("Working with: oo: {:?}, side: {:?}, price: {}, mbq: {}, mqq: {}, client id: {}, type: {:?}, reduce: {}, et: {:?}, max: {}",
    //     [Pubkey::default();0],
    //     side,
    //     price,
    //     max_base_quantity,
    //     max_quote_quantity,
    //     client_order_id,
    //     OrderType::Limit,  // order_type
    //     false,  // reduce_only
    //     0,  // expiry_timestamp, 0 to ignore time in force
    //     20,  //limit: no limit, max 8byte unsigned
    // );

    let instruction = mango::instruction::place_perp_order2(
        ctx.accounts.mango_program.key,
        ctx.accounts.mango_group.key,
        ctx.accounts.mango_account.key,
        ctx.accounts.authority.key,  // owner_pk
        ctx.accounts.mango_cache.key,
        ctx.accounts.perp_market.key,
        ctx.accounts.mango_bids.key,
        ctx.accounts.mango_asks.key,
        ctx.accounts.mango_event_queue.key,
        None,  // referrer_mango_account_pk, consider to work with this later
        &[Pubkey::default();0], // TODO: open orders seems being empty for perp orders, probably necessary for spot, the remaining accounts for context needs to be processed here
        side,
        price,
        max_base_quantity,
        max_quote_quantity,
        client_order_id,
        OrderType::Limit,  // order_type, TODO: hardcoding limit here right now
        false,  // reduce_only
        None,  // expiry_timestamp, 0 to ignore time in force
        20,  //limit: no limit, max 8byte unsigned, TODO: hardcoding something here right now
    ).unwrap();
    msg!("Going to invoke instruction: {:?}", instruction);
    let account_infos_vec = vec![
        ctx.accounts.mango_group.clone(),
        ctx.accounts.mango_account.clone(),
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.mango_cache.clone(),
        ctx.accounts.perp_market.clone(),
        ctx.accounts.mango_bids.clone(),
        ctx.accounts.mango_asks.clone(),
        ctx.accounts.mango_event_queue.clone(),
        ctx.accounts.mango_account.clone(),  // referall, curently just the mango account
    ];
    // account_infos_vec.append(&mut ctx.remaining_accounts.clone().to_vec());
    msg!("Accounts to be used: {:?}", account_infos_vec);
    solana_program::program::invoke(
        &instruction,
        &account_infos_vec.as_slice(),
    ).unwrap();
}

pub fn cancel_all_perp_orders(ctx: &Context<CancelAll>) {
    let ix = mango::instruction::cancel_all_perp_orders(
        ctx.accounts.mango_program.key,
        ctx.accounts.mango_group.key,
        ctx.accounts.mango_account.key,
        ctx.accounts.authority.key,
        ctx.accounts.perp_market.key,
        ctx.accounts.mango_bids.key,
        ctx.accounts.mango_asks.key,
        20).unwrap();
    msg!("Instruction is {:?}", ix);
    let account_infos = [
        ctx.accounts.mango_program.clone(),
        ctx.accounts.mango_group.clone(),
        ctx.accounts.mango_account.clone(),
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.perp_market.clone(),
        ctx.accounts.mango_bids.clone(),
        ctx.accounts.mango_asks.clone()
    ];
    solana_program::program::invoke(&ix, &account_infos).unwrap()
}

pub fn cancel_perp_order_by_client_id(
    ctx: &Context<CancelPerpOrderByClientId>,
    client_order_id: u64,
    invalid_id_ok: bool
) {
    let ix = mango::instruction::cancel_perp_order_by_client_id(
        ctx.accounts.mango_program.key,
        ctx.accounts.mango_group.key,
        ctx.accounts.mango_account.key,
        ctx.accounts.authority.key,
        ctx.accounts.perp_market.key,
        ctx.accounts.mango_bids.key,
        ctx.accounts.mango_asks.key,
        client_order_id,
        invalid_id_ok,
    ).unwrap();
    msg!("Instruction is {:?}", ix);
    let account_infos = [
        ctx.accounts.mango_program.clone(),
        ctx.accounts.mango_group.clone(),
        ctx.accounts.mango_account.clone(),
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.perp_market.clone(),
        ctx.accounts.mango_bids.clone(),
        ctx.accounts.mango_asks.clone()
    ];
    solana_program::program::invoke(&ix, &account_infos).unwrap()
}

pub fn place_perp_order_proxy<'info>(
    ctx: &Context<'_, '_, '_, 'info, PlaceOrder<'info>>,
    client_order_id: u64,
    side: Side,
    price: i64,
    max_base_quantity: i64,
    max_quote_quantity: i64,
) {
    let remaining_accounts_iter = ctx.remaining_accounts.iter();
    let mut open_orders_accounts: Vec<AccountInfo> = vec![];
    remaining_accounts_iter.for_each(|ai| open_orders_accounts.push(ai.clone()));

    let cpi_accounts = PlacePerpOrder2 {
        mango_group: ctx.accounts.mango_group.clone(),
        mango_account: ctx.accounts.mango_account.clone(),
        owner: ctx.accounts.authority.to_account_info(),
        mango_cache:ctx.accounts.mango_cache.clone(),
        perp_market: ctx.accounts.perp_market.clone(),
        bids: ctx.accounts.mango_bids.clone(),
        asks: ctx.accounts.mango_asks.clone(),
        event_queue: ctx.accounts.mango_event_queue.clone(),
        referral: ctx.accounts.mango_account.clone()
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.mango_program.clone(),
        cpi_accounts
    ).with_remaining_accounts(open_orders_accounts);
    let result:Result<()> = place_perp_order2(
        cpi_ctx,
        side,
        price,
        max_base_quantity,
        max_quote_quantity,
        client_order_id,
        OrderType::Limit,
        false,  // reduce only
        None,  // do not define any delegate account
        u8::MAX,
    );
    match result {
        Ok(t) => msg!("All fine: {:?}", t),
        Err(e) => {
            msg!("Failure on processing: {:?}", e);
        }
    };
}

#[derive(Accounts)]
#[instruction(market_index: u8)]
pub struct Create<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 8 + 1,
        seeds = [SEED_PHRASE, [market_index].as_ref(), authority.key().as_ref()],
        bump
    )]
    pub pda_market_account: Account<'info, MarketContractAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(client_order_id: u64, side: u8, price: i64, max_base_quantity: i64, max_quote_quantity: i64)]
pub struct PlaceOrder<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority,
        seeds = [SEED_PHRASE, [pda_market_account.market_index].as_ref(), authority.key().as_ref()],
        bump = pda_market_account.bump
    )]
    pub pda_market_account: Box<Account<'info, MarketContractAccount>>,
    /// CHECK: read-only
    pub mango_program: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub mango_account: AccountInfo<'info>,
    /// CHECK: read-only
    pub mango_group: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub perp_market: AccountInfo<'info>,
    /// CHECK: read-only
    pub mango_cache: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub mango_bids: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub mango_asks: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub mango_event_queue: AccountInfo<'info>,

    // OpenOrders: need to be added to
    // CpiContext's `remaining_accounts` Vec [0-MAX_PAIRS]
}

#[derive(Accounts)]
pub struct CancelAll<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority,
        seeds = [SEED_PHRASE, [pda_market_account.market_index].as_ref(), authority.key().as_ref()],
        bump = pda_market_account.bump
    )]
    pub pda_market_account: Box<Account<'info, MarketContractAccount>>,
    /// CHECK: read-only
    pub mango_program: AccountInfo<'info>,
    /// CHECK: read-only
    pub mango_group: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub mango_account: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub perp_market: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub mango_bids: AccountInfo<'info>,
    /// CHECK
    #[account(mut)]
    pub mango_asks: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(client_order_id: u64, invalid_id_ok: bool)]
pub struct CancelPerpOrderByClientId<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority,
        seeds = [SEED_PHRASE, [pda_market_account.market_index].as_ref(), authority.key().as_ref()],
        bump = pda_market_account.bump
    )]
    pub pda_market_account: Box<Account<'info, MarketContractAccount>>,
    /// CHECK: read-only
    pub mango_program: AccountInfo<'info>,
    /// CHECK: read-only
    pub mango_group: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub mango_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub perp_market: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub mango_bids: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub mango_asks: AccountInfo<'info>
}

#[account]
#[derive(Default)]
pub struct MarketContractAccount {
    pub authority: Pubkey,  // 32
    pub bump: u8, // 1
    pub counter: u64, // 8
    pub market_index: u8, // 1
}
