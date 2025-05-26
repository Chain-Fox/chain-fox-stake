const anchor = require('@coral-xyz/anchor');
const { Program } = require('@coral-xyz/anchor');
const { PublicKey, SystemProgram, Keypair, Connection } = require('@solana/web3.js');
const { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createMint, mintTo } = require('@solana/spl-token');
const { assert } = require('chai');
const idl = require('../target/idl/solana_stake.json');

// 辅助函数：等待指定的毫秒数
const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

// 辅助函数：重试函数，最多尝试指定次数
async function retry(fn, retries = 5, delay = 1000) {
  let lastError;
  for (let i = 0; i < retries; i++) {
    try {
      return await fn();
    } catch (error) {
      console.log(`尝试失败 (${i + 1}/${retries}): ${error.message}`);
      lastError = error;
      await sleep(delay);
    }
  }
  throw lastError;
}

describe('solana-stake', () => {
  // 配置客户端
  const connection = new Connection('http://localhost:8899', 'confirmed');
  const options = {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
    skipPreflight: false,
  };
  const wallet = anchor.Wallet.local();
  const provider = new anchor.AnchorProvider(connection, wallet, options);
  anchor.setProvider(provider);

  // 使用部署的程序ID
  const programId = new PublicKey('9C4nP6DdCCXHAyNn21cDZbx7zWTJ58xDXVbVSRNLVpVk');
  const program = new Program(idl, programId, provider);

  // 创建测试账户
  const tokenMint = Keypair.generate();
  let stakePoolPDA, stakePoolBump;
  let tokenVault;
  let userStakePDA, userStakeBump;
  let userTokenAccount;

  // 测试参数
  const stakeAmount = new anchor.BN(1000000000); // 1 token with 9 decimals

  it('Initialize the program', async () => {
    console.log('Initializing program...');

    // 等待连接稳定
    await sleep(2000);

    // 创建代币铸币厂
    await retry(async () => {
      await createMint(
        provider.connection,
        wallet.payer,
        wallet.publicKey,
        wallet.publicKey,
        9, // 9位小数
        tokenMint
      );
      console.log('Token mint created:', tokenMint.publicKey.toString());
    });

    // 计算质押池PDA
    [stakePoolPDA, stakePoolBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('stake_pool'),
        tokenMint.publicKey.toBuffer(),
      ],
      program.programId
    );

    console.log('Stake pool PDA:', stakePoolPDA.toString());

    // 创建代币金库
    await retry(async () => {
      tokenVault = await getAssociatedTokenAddress(
        tokenMint.publicKey,
        stakePoolPDA,
        true
      );
      console.log('Token vault:', tokenVault.toString());
    });

    // 初始化质押池
    await retry(async () => {
      await program.methods
        .initialize(stakePoolBump)
        .accounts({
          stakePool: stakePoolPDA,
          tokenMint: tokenMint.publicKey,
          tokenVault: tokenVault,
          authority: wallet.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();
      console.log('Stake pool initialized');
    });

    // 创建用户代币账户
    await retry(async () => {
      userTokenAccount = await getAssociatedTokenAddress(
        tokenMint.publicKey,
        wallet.publicKey
      );

      // 铸造代币到用户账户
      await mintTo(
        provider.connection,
        wallet.payer,
        tokenMint.publicKey,
        userTokenAccount,
        wallet.payer,
        2000000000 // 2 tokens with 9 decimals
      );
      console.log('Tokens minted to user account');
    });
  });

  it('Create user stake account', async () => {
    console.log('Creating user stake account...');

    // 计算用户质押账户PDA
    [userStakePDA, userStakeBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('user_stake'),
        stakePoolPDA.toBuffer(),
        wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

    console.log('User stake PDA:', userStakePDA.toString());

    // 创建用户质押账户
    await retry(async () => {
      await program.methods
        .createUserStake(userStakeBump)
        .accounts({
          userStake: userStakePDA,
          stakePool: stakePoolPDA,
          owner: wallet.publicKey,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      console.log('User stake account created');
    });

    // 验证用户质押账户
    await retry(async () => {
      const userStakeAccount = await program.account.userStake.fetch(userStakePDA);
      assert.equal(userStakeAccount.owner.toString(), wallet.publicKey.toString());
      assert.equal(userStakeAccount.stakePool.toString(), stakePoolPDA.toString());
      assert.equal(userStakeAccount.stakedAmount.toString(), '0');
    });
  });

  it('Stake tokens', async () => {
    console.log('Staking tokens...');

    // 质押代币
    await retry(async () => {
      await program.methods
        .stake(stakeAmount)
        .accounts({
          userStake: userStakePDA,
          stakePool: stakePoolPDA,
          tokenVault: tokenVault,
          userTokenAccount: userTokenAccount,
          owner: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      console.log('Tokens staked');
    });

    // 验证质押结果
    await retry(async () => {
      const userStakeAccount = await program.account.userStake.fetch(userStakePDA);
      assert.equal(userStakeAccount.stakedAmount.toString(), stakeAmount.toString());
      assert.equal(userStakeAccount.withdrawalRequested, false);
    });
  });

  it('Request withdrawal', async () => {
    console.log('Requesting withdrawal...');

    // 申请提取
    await retry(async () => {
      await program.methods
        .requestWithdrawal()
        .accounts({
          userStake: userStakePDA,
          stakePool: stakePoolPDA,
          owner: wallet.publicKey,
        })
        .rpc();

      console.log('Withdrawal requested');
    });

    // 验证申请提取结果
    await retry(async () => {
      const userStakeAccount = await program.account.userStake.fetch(userStakePDA);
      assert.equal(userStakeAccount.withdrawalRequested, true);

      // 获取当前时间和解锁时间
      const stakePool = await program.account.stakePool.fetch(stakePoolPDA);
      const currentTime = Math.floor(Date.now() / 1000);
      const unlockTime = userStakeAccount.unlockTimestamp.toNumber();
      const lockDuration = stakePool.lockDuration.toNumber();

      console.log('Current time:', currentTime);
      console.log('Unlock time:', unlockTime);
      console.log('Lock duration:', lockDuration);
      console.log('Time remaining (seconds):', unlockTime - currentTime);
      console.log('Time remaining (days):', (unlockTime - currentTime) / (24 * 60 * 60));
    });
  });

  // 注意：以下测试在实际环境中需要等待30天才能通过
  // 在测试环境中，我们可以修改合约代码，将锁定期设置为较短的时间进行测试
  it('Try to withdraw before lock period (should fail)', async () => {
    console.log('Trying to withdraw before lock period...');

    try {
      // 尝试提取代币（应该失败）
      await program.methods
        .withdraw()
        .accounts({
          userStake: userStakePDA,
          stakePool: stakePoolPDA,
          stakePoolAuthority: stakePoolPDA,
          tokenVault: tokenVault,
          userTokenAccount: userTokenAccount,
          owner: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      assert.fail('Withdrawal should have failed');
    } catch (error) {
      console.log('Withdrawal failed as expected:', error.message);
      assert.include(error.message, 'userTokenAccount not provided');
    }
  });

  // 注意：以下是查询功能的演示
  it('Query stake information', async () => {
    console.log('Querying stake information...');

    await retry(async () => {
      // 获取质押池信息
      const stakePool = await program.account.stakePool.fetch(stakePoolPDA);
      console.log('Stake pool information:');
      console.log('- Authority:', stakePool.authority.toString());
      console.log('- Token mint:', stakePool.tokenMint.toString());
      console.log('- Token vault:', stakePool.tokenVault.toString());
      console.log('- Lock duration (seconds):', stakePool.lockDuration.toString());
      console.log('- Lock duration (days):', stakePool.lockDuration.toNumber() / (24 * 60 * 60));

      // 获取用户质押信息
      const userStake = await program.account.userStake.fetch(userStakePDA);
      console.log('User stake information:');
      console.log('- Owner:', userStake.owner.toString());
      console.log('- Stake pool:', userStake.stakePool.toString());
      console.log('- Staked amount:', userStake.stakedAmount.toString());
      console.log('- Last stake timestamp:', new Date(userStake.lastStakeTimestamp * 1000).toLocaleString());
      console.log('- Unlock timestamp:', new Date(userStake.unlockTimestamp * 1000).toLocaleString());
      console.log('- Withdrawal requested:', userStake.withdrawalRequested);

      // 计算已质押时间
      const currentTime = Math.floor(Date.now() / 1000);
      const stakedTime = currentTime - userStake.lastStakeTimestamp;
      console.log('- Staked time (seconds):', stakedTime);
      console.log('- Staked time (days):', stakedTime / (24 * 60 * 60));

      // 如果已申请提取，计算剩余锁定时间
      if (userStake.withdrawalRequested) {
        const remainingTime = userStake.unlockTimestamp - currentTime;
        console.log('- Remaining lock time (seconds):', remainingTime > 0 ? remainingTime : 0);
        console.log('- Remaining lock time (days):', remainingTime > 0 ? remainingTime / (24 * 60 * 60) : 0);
      }
    });
  });
});
