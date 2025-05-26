
const anchor = require("@coral-xyz/anchor");
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");

describe("chain-fox-dao-initialize", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  it("Initialize Stake Pool", async () => {
    const program = anchor.workspace.CfxStakeCore;
    
    // 计算 PDA
    const [stakePoolPDA, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("stake_pool"), new PublicKey("6trEEfEZ7LPuhFDjXt7jr4FubXXhovViKnfFrGUFNhG4").toBuffer()],
      program.programId
    );
    
    // 创建代币金库
    const tokenVault = anchor.web3.Keypair.generate();
    
    try {
      await program.methods
        .initialize(bump, null, null)
        .accounts({
          stakePool: stakePoolPDA,
          tokenMint: new PublicKey("6trEEfEZ7LPuhFDjXt7jr4FubXXhovViKnfFrGUFNhG4"),
          tokenVault: tokenVault.publicKey,
          authority: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([tokenVault])
        .rpc();
      
      console.log("✅ 质押池初始化成功");
    } catch (error) {
      if (error.message.includes("already in use")) {
        console.log("✅ 质押池已经初始化");
      } else {
        throw error;
      }
    }
  });
});
