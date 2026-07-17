use anchor_lang::prelude::*;

declare_id!("BiqsXC7Uqskmp5ucSmfUww6bwVqNVnAYEE2FGvifD8kS");

#[program]
pub mod myla_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
