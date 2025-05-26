// Chain-Fox DAO 租金计算脚本
// 用于计算 cfx-stake-core、cfx-rewards 和 cfx-liquidity 三个程序的部署租金

const anchor = require("@coral-xyz/anchor");
const { PublicKey, LAMPORTS_PER_SOL } = require("@solana/web3.js");
const fs = require('fs');
const path = require('path');

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

// 计算所有程序的租金
async function calculateTotalRent(connection) {
  console.log(`${colors.yellow}计算所有程序的租金...${colors.reset}`);
  console.log();

  let totalRent = 0;
  const programDetails = [];

  for (const program of PROGRAMS) {
    const programSize = getProgramSize(program.name);
    if (programSize === 0) continue;

    const rentAmount = await calculateRent(connection, programSize);
    totalRent += rentAmount;

    programDetails.push({
      name: program.name,
      size: programSize,
      rent: rentAmount
    });
  }

  if (programDetails.length === 0) {
    console.log(`${colors.red}错误: 没有找到有效的程序文件${colors.reset}`);
    return { totalRent: 0, programDetails: [] };
  }

  // 打印租金信息表格
  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log(`${colors.blue}                    程序租金信息                         ${colors.reset}`);
  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log(`${colors.blue}程序名称               大小 (字节)      租金 (SOL)${colors.reset}`);
  console.log(`${colors.blue}---------------------------------------------------------${colors.reset}`);

  for (const program of programDetails) {
    console.log(`${program.name.padEnd(24)}${program.size.toString().padEnd(18)}${program.rent.toFixed(6)} SOL`);
  }

  console.log(`${colors.blue}---------------------------------------------------------${colors.reset}`);
  console.log(`${'总计'.padEnd(24)}${' '.padEnd(18)}${totalRent.toFixed(6)} SOL`);
  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log();

  return { totalRent, programDetails };
}

// 主函数
async function main() {
  // 获取命令行参数中的集群
  const args = process.argv.slice(2);
  const clusterArg = args.find(arg => arg.startsWith('--cluster='));
  const cluster = clusterArg ? clusterArg.split('=')[1] : 'localnet';

  // 根据集群参数设置连接
  let url;
  switch (cluster) {
    case 'mainnet':
      url = 'https://api.mainnet-beta.solana.com';
      break;
    case 'devnet':
      url = 'https://api.devnet.solana.com';
      break;
    case 'localnet':
    default:
      url = 'http://localhost:8899';
  }

  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log(`${colors.blue}        Chain-Fox DAO 程序租金计算工具                   ${colors.reset}`);
  console.log(`${colors.blue}     (cfx-stake-core, cfx-rewards, cfx-liquidity)        ${colors.reset}`);
  console.log(`${colors.blue}==========================================================${colors.reset}`);
  console.log();
  console.log(`${colors.yellow}集群: ${cluster}${colors.reset}`);
  console.log(`${colors.yellow}RPC URL: ${url}${colors.reset}`);
  console.log();

  try {
    // 创建连接
    const connection = new anchor.web3.Connection(url, 'confirmed');

    // 测试连接
    console.log(`${colors.yellow}正在连接到 Solana 节点...${colors.reset}`);
    await connection.getVersion();
    console.log(`${colors.green}连接成功！${colors.reset}`);
    console.log();

    // 计算租金
    await calculateTotalRent(connection);
  } catch (error) {
    console.log();
    console.log(`${colors.red}错误: 无法连接到 ${cluster} 集群${colors.reset}`);

    if (cluster === 'localnet') {
      console.log(`${colors.yellow}本地验证节点可能没有运行。请尝试以下操作:${colors.reset}`);
      console.log(`${colors.yellow}1. 启动本地验证节点: solana-test-validator${colors.reset}`);
      console.log(`${colors.yellow}2. 或者使用开发网: node migrations/calculate-rent.js --cluster=devnet${colors.reset}`);
    } else if (cluster === 'devnet') {
      console.log(`${colors.yellow}无法连接到开发网。请检查您的网络连接或稍后再试。${colors.reset}`);
    } else if (cluster === 'mainnet') {
      console.log(`${colors.yellow}无法连接到主网。请检查您的网络连接或稍后再试。${colors.reset}`);
    }

    console.log();
    console.log(`${colors.red}详细错误信息:${colors.reset}`);
    console.log(error);
    process.exit(1);
  }
}

// 如果直接运行此脚本，则执行主函数
if (require.main === module) {
  main().catch(err => {
    console.error(`${colors.red}未处理的错误:${colors.reset}`, err);
    process.exit(1);
  });
}

module.exports = { calculateTotalRent, calculateRent, getProgramSize };
