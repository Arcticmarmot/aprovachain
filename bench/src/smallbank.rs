use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ledger::call::LedgerCall;
use platform::config::load_base_config;

#[derive(Debug, Clone)]
pub struct WorkloadCfg {
    pub account_num: usize,
    pub tx_num: usize,
    /// 热点集合占比：0.01 表示 1% 账户是热点
    pub hot_ratio: f64,
    /// 选账户时落在热点集合的概率：0.90 表示 90% 的访问打到热点
    pub p_hot: f64,
    /// 四类操作比例（百分比，加起来必须 = 100）
    pub pct_transfer: u32,
    pub pct_query: u32,
    pub pct_mint: u32,
    pub pct_burn: u32,
    /// 金额范围（先用最简单的均匀分布）
    pub min_amount: u64,
    pub max_amount: u64,
    /// 是否先给每个账户 Mint 一笔初始余额（让 Burn/Transfer 更不容易失败）
    pub with_init_mint: bool,
}

/// 生成 simplified smallbank 数据集：输出 Vec<LedgerCall>
pub fn gen_simplified_smallbank(
    addrs: &[String],
    cfg: &WorkloadCfg,
    seed: u64,
) -> Vec<LedgerCall> {
    assert!(cfg.account_num > 0);
    assert!(addrs.len() >= cfg.account_num);
    assert!(cfg.min_amount <= cfg.max_amount);

    let sum = cfg.pct_transfer + cfg.pct_query + cfg.pct_mint + cfg.pct_burn;
    assert_eq!(sum, 100, "pct_* must sum to 100, got {sum:?}");

    let n = cfg.account_num;
    let hot_size = ((cfg.hot_ratio * n as f64).round() as usize).clamp(1, n);

    let mut rng = StdRng::seed_from_u64(seed);

    // 账户选择：hotset 混合
    let pick_account = |rng: &mut StdRng| -> usize {
        let roll: f64 = rng.random_range(0.0..1.0); // [0, 1)
        if roll < cfg.p_hot {
            rng.random_range(0..hot_size)
        } else {
            rng.random_range(0..n)
        }
    };

    let mut out = Vec::with_capacity(cfg.tx_num + if cfg.with_init_mint { n } else { 0 });

    for _ in 0..cfg.tx_num {
        let amount = rng.random_range(cfg.min_amount..=cfg.max_amount);
        let op_roll = rng.random_range(0..100);

        let call = if op_roll < cfg.pct_transfer {
            let from = pick_account(&mut rng);
            let mut to = pick_account(&mut rng);
            if to == from {
                to = (to + 1) % n;
            }
            LedgerCall::Transfer {
                from: addrs[from].clone(),
                to: addrs[to].clone(),
                amount,
            }
        } else if op_roll < cfg.pct_transfer + cfg.pct_query {
            let a = pick_account(&mut rng);
            LedgerCall::QueryBalance { addr: addrs[a].clone() }
        } else if op_roll < cfg.pct_transfer + cfg.pct_query + cfg.pct_mint {
            let a = pick_account(&mut rng);
            LedgerCall::Mint { to: addrs[a].clone(), amount }
        } else {
            let a = pick_account(&mut rng);
            LedgerCall::Burn { from: addrs[a].clone(), amount }
        };

        out.push(call);
    }

    out
}



/// 两套配置：Low / High conflict（先按这个跑通）
pub fn workload_low() -> WorkloadCfg {
    let base = load_base_config();
    let workload = base.workload;
    WorkloadCfg {
        account_num: 1000,
        tx_num: workload.tx_num,
        hot_ratio: 0.10,
        p_hot: 0.20,
        pct_transfer: 25,
        pct_query: 50,
        pct_mint: 15,
        pct_burn: 10,
        min_amount: 1,
        max_amount: 100,
        with_init_mint: true,
    }
}

pub fn workload_high() -> WorkloadCfg {
    let base = load_base_config();
    let workload = base.workload;
    WorkloadCfg {
        account_num: workload.accounts_num,
        tx_num: workload.tx_num,
        hot_ratio: 0.01,
        p_hot: 0.90,
        pct_transfer: 40,
        pct_query: 40,
        pct_mint: 10,
        pct_burn: 10,
        min_amount: 1,
        max_amount: 100,
        with_init_mint: true,
    }
}
