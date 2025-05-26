// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

// Chain-Fox DAO 拆分程序部署脚本
// 用于部署 cfx-stake-core、cfx-rewards 和 cfx-liquidity 三个程序
//
// 使用方法：
// - 部署程序: anchor migrate
// - 计算租金: node migrations/calculate-rent.js --cluster=localnet

const anchor = require("@coral-xyz/anchor");
const { PublicKey, LAMPORTS_PER_SOL } = require("@solana/web3.js");
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// 程序信息
const PROGRAMS = [
  {
    name: 'cfx_stake_core',
    dirName: 'cfx-stake-core',
    id: '426MdbCio9rvekWxFiz2AmEQwBXAkASZqmrf3eW1RQAo',
  },
  {
    name: 'cfx_rewards',
    dirName: 'cfx-rewards',
    id: 'BgWUGrXRKF3pgVEgstwau11AGgynhsZwyiHhXoC5bn6t',
  },
  {
    name: 'cfx_liquidity',
    dirName: 'cfx-liquidity',
    id: '3Hn6Smh85GBpwWdAvu4sCgg5TjsQtUsuAsYp5t4yyqKn',
  }
];

// 颜色定义
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
};

// 获取程序文件大小
function getProgramSize(programName) {
  const programPath = path.join(__dirname, '..', 'target', 'deploy', `${programName}.so`);

  if (!fs.existsSync(programPath)) {
    console.log(`${colors.red}错误: 程序文件不存在: ${programPath}${colors.reset}`);
    return 0;
  }

  const stats = fs.statSync(programPath);
  return stats.size;
}

// 计算程序租金
async function calculateRent(connection, programSize) {
  if (programSize === 0) return 0;

  // 计算程序所需的租金豁免金额
  const rentExemptionAmount = await connection.getMinimumBalanceForRentExemption(programSize);
  return rentExemptionAmount / LAMPORTS_PER_SOL;
}

// 计算所有程序的总租金
async function calculateTotalRent(connection) {
  let totalRent = 0;

  for (const program of PROGRAMS) {
    const programSize = getProgramSize(program.name);
    if (programSize === 0) continue;

    const rentAmount = await calculateRent(connection, programSize);
    totalRent += rentAmount;
  }

  return { totalRent };
}

// 获取实际部署的程序 ID
function getActualProgramId(programName) {
  const keypairPath = path.join(__dirname, '..', 'target', 'deploy', `${programName}-keypair.json`);

  if (!fs.existsSync(keypairPath)) {
    console.log(`${colors.red}警告: 找不到程序密钥文件: ${keypairPath}${colors.reset}`);
    return null;
  }

  try {
    const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
    const keypair = anchor.web3.Keypair.fromSecretKey(new Uint8Array(keypairData));
    return keypair.publicKey.toString();
  } catch (error) {
    console.log(`${colors.red}错误: 无法读取程序密钥文件 ${keypairPath}: ${error.message}${colors.reset}`);
    return null;
  }
}

// 保存程序 ID 到配置文件
async function saveProgramIds() {
  console.log(`${colors.yellow}保存程序 ID 到配置文件...${colors.reset}`);

  const programIds = {};
  const deployedPrograms = [];

  // 获取所有实际部署的程序 ID
  for (const program of PROGRAMS) {
    const actualId = getActualProgramId(program.name);
    if (actualId) {
      programIds[program.name.toUpperCase()] = actualId;
      deployedPrograms.push({
        name: program.name,
        id: actualId,
        dirName: program.dirName
      });
      console.log(`${colors.green}  ${program.name}: ${actualId}${colors.reset}`);
    } else {
      console.log(`${colors.red}  ${program.name}: 获取失败${colors.reset}`);
    }
  }

  // 创建配置对象
  const config = {
    timestamp: new Date().toISOString(),
    network: "localnet", // 可以根据实际网络调整
    deployer: "unknown", // 可以在调用时传入
    programs: programIds,
    deployed_programs: deployedPrograms,
    metadata: {
      total_programs: PROGRAMS.length,
      successful_deployments: deployedPrograms.length,
      deployment_tool: "anchor",
      version: "1.0.0"
    }
  };

  // 保存到文件
  const configPath = path.join(__dirname, '..', 'program-ids.json');
  try {
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
    console.log(`${colors.green}程序 ID 配置已保存到: ${configPath}${colors.reset}`);
    console.log(`${colors.blue}其他脚本可以通过读取此文件获取最新的程序 ID${colors.reset}`);
  } catch (error) {
    console.log(`${colors.red}保存程序 ID 配置失败: ${error.message}${colors.reset}`);
  }
}



// 部署脚本
module.exports = async function (provider) {
  // 配置 client 使用提供的 provider
  anchor.setProvider(provider);

  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log(`${colors.blue}        Chain-Fox DAO 拆分程序部署工具                   ${colors.reset}`);
  console.log(`${colors.blue}     (cfx-stake-core, cfx-rewards, cfx-liquidity)        ${colors.reset}`);
  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log();

  // 获取部署者的公钥
  const deployer = provider.wallet.publicKey;
  console.log(`${colors.yellow}部署者地址: ${deployer.toString()}${colors.reset}`);

  // 获取钱包余额
  const walletBalance = await provider.connection.getBalance(deployer) / LAMPORTS_PER_SOL;
  console.log(`${colors.yellow}钱包余额: ${walletBalance.toFixed(6)} SOL${colors.reset}`);
  console.log();

  // 计算租金
  const { totalRent } = await calculateTotalRent(provider.connection);

  // 检查余额是否足够
  const balanceSufficient = walletBalance >= totalRent;
  if (!balanceSufficient) {
    console.log(`${colors.red}警告: 钱包余额不足以支付所有程序的租金！${colors.reset}`);
    console.log(`${colors.red}需要至少 ${totalRent.toFixed(6)} SOL，但钱包中只有 ${walletBalance.toFixed(6)} SOL${colors.reset}`);
    console.log(`${colors.red}建议先充值足够的 SOL 再继续。${colors.reset}`);
    console.log();

    // 提供查看详细租金信息的提示
    console.log(`${colors.yellow}如需查看详细的租金信息，请运行: node migrations/calculate-rent.js${colors.reset}`);
    console.log();
  } else {
    console.log(`${colors.green}钱包余额足够支付所有程序的租金${colors.reset}`);
    console.log();
  }

  // 自动部署程序
  console.log(`${colors.yellow}开始部署所有程序...${colors.reset}`);
  console.log();

  let successCount = 0;
  let failureCount = 0;

  // 部署每个程序
  for (const program of PROGRAMS) {
    console.log(`${colors.yellow}正在部署 ${program.name}...${colors.reset}`);

    const programPath = path.join(__dirname, '..', 'target', 'deploy', `${program.name}.so`);

    if (!fs.existsSync(programPath)) {
      console.log(`${colors.red}错误: 程序文件不存在: ${programPath}${colors.reset}`);
      failureCount++;
      continue;
    }

    const programSize = getProgramSize(program.name);
    console.log(`${colors.blue}程序大小: ${programSize} 字节${colors.reset}`);

    const rentAmount = await calculateRent(provider.connection, programSize);
    console.log(`${colors.blue}程序租金: ${rentAmount.toFixed(6)} SOL${colors.reset}`);

    try {
      // 使用 Anchor 部署程序
      console.log(`${colors.yellow}使用 Anchor 部署程序...${colors.reset}`);

      // 执行部署命令
      // 使用 provider 中的集群配置
      const cluster = provider.connection._rpcEndpoint.includes('localhost') ? 'localnet' :
                     provider.connection._rpcEndpoint.includes('devnet') ? 'devnet' :
                     provider.connection._rpcEndpoint.includes('mainnet') ? 'mainnet' :
                     'localnet';

      const deployCommand = `anchor deploy --program-name ${program.name} --provider.cluster ${cluster}`;
      console.log(`${colors.blue}执行命令: ${deployCommand}${colors.reset}`);

      execSync(deployCommand, { stdio: 'inherit' });

      console.log(`${colors.green}程序 ${program.name} 部署成功！${colors.reset}`);
      console.log(`${colors.green}程序 ID: ${program.id}${colors.reset}`);
      successCount++;
    } catch (error) {
      console.log(`${colors.red}程序 ${program.name} 部署失败: ${error.message}${colors.reset}`);
      failureCount++;
    }

    console.log();
    console.log(`${colors.blue}--------------------------------------------------------${colors.reset}`);
    console.log();
  }

  // 打印部署结果摘要
  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log(`${colors.blue}                    部署结果摘要                         ${colors.reset}`);
  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log(`${colors.green}成功部署的程序数量: ${successCount}${colors.reset}`);
  console.log(`${colors.red}失败部署的程序数量: ${failureCount}${colors.reset}`);
  console.log();

  if (failureCount === 0) {
    console.log(`${colors.green}所有程序部署成功！${colors.reset}`);

    // 保存程序 ID 到配置文件
    await saveProgramIds();
  } else {
    console.log(`${colors.red}部分程序部署失败，请检查错误信息。${colors.reset}`);
  }
}
