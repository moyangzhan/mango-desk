pub fn finalize_metadata_embedding(raw_weights: Vec<f32>, target_dim: usize) -> Vec<f32> {
    // 1. 物理截断 (Matryoshka Truncation)
    let mut truncated = raw_weights;
    truncated.truncate(target_dim);

    // 2. 计算 L2 范数 (Norm)
    let squared_sum: f32 = truncated.iter().map(|x| x * x).sum();
    let norm = squared_sum.sqrt();

    // 3. 执行归一化 (防止除以 0)
    if norm > f32::EPSILON {
        truncated.into_iter().map(|x| x / norm).collect()
    } else {
        truncated
    }
}

pub fn calculate_content_score(distance: f32, sparse_score: f32, is_short_query: bool) -> usize {
    let dense_sim = (1.0 - distance.min(1.0)).max(0.0).min(1.0);

    let hybrid_score = if is_short_query {
        // 1. 如果距离太远 (distance > 0.9)，说明语义完全不通
        if dense_sim < 0.1 {
            return (sparse_score * 50.0) as usize; // 强制降级到 10分以下，直接过滤
        }
        // --- 短查询模式：严格执行“关键词匹配”校验 ---
        if sparse_score > 0.15 {
            // 关键词命中 (Exact/Strong Match)
            0.5 + (dense_sim * 0.5)
        } else if dense_sim > 0.65 {
            // 【补偿逻辑】：即便 Sparse 只有 0.03，但如果语义距离 < 0.35 (sim > 0.65)
            // 说明这极有可能是类似 "monster -> beast" 的高质量同义词或隐喻
            // 给予足够的分数通过 40 分门槛
            0.3 + (dense_sim * 0.35)
            // 计算：0.3 + (0.7 * 0.35) = 0.54 (54分) -> 成功召回！
        } else {
            // 既没中词，语义距离又远 (如 killer -> rust)
            dense_sim * 0.2
        }
    } else {
        // --- 长查询模式：弱约束 ---
        if sparse_score > 0.15 {
            // 情况 A：长句中有多个核心词精准命中
            // 此时我们 100% 信任 Dense 分数，甚至可以给一个 5 分的“确信奖”
            (dense_sim * 1.0 + 0.05).min(1.0)
        } else if sparse_score > 0.02 {
            // 情况 B：有一定的词汇重合，但不算强
            // 直接使用 Dense 分数，不做任何奖惩
            dense_sim
        } else {
            // 情况 C：语义很近，但【一个词】都没对上 (Sparse < 0.02)
            // 这种情况在长查询中比较可疑（可能是意境图/幻觉）
            // 给予轻微惩罚，降权 15%
            dense_sim * 0.85
        }
    };

    (hybrid_score * 100.0).round().clamp(0.0, 100.0) as usize
}

pub fn calculate_metadata_score(distance: f32, sparse_score: f32) -> usize {
    // 1. 将余弦距离转换为相似度 (0.0 到 1.0)
    let dense_sim = (1.0 - distance.min(1.0)).max(0.0).min(1.0);

    //1. 如果距离太远 (distance > 0.9)，说明语义完全不通
    if dense_sim < 0.1 {
        return (sparse_score * 50.0) as usize; // 强制降级到 10分以下，直接过滤
    }

    // 3. 核心加权逻辑：0.2 Dense + 0.8 Sparse
    // 这里的逻辑是：即便语义有点像 (Dense)，如果关键词一个没对上 (Sparse=0)，分值会极低
    let hybrid_score = if sparse_score < 0.01 {
        // --- 惩罚项：关键词未命中 ---
        // 即使向量很近，如果没有关键词对齐，得分大幅衰减
        dense_sim * 0.1
    } else {
        // --- 激励项：标题关键词命中 ---
        // 1. 基础奖励 (Base Boost): 只要标题里有这个词，起步给 0.4 (40分)
        let base_boost = 0.4;

        // 2. 词法平滑 (Sparse Smoothing):
        // 使用 tanh 快速放大低分段（针对 puppy 这种 0.029 的情况）
        // 乘以 20.0 是为了让 0.03 左右的微弱分值快速冲向 0.5 左右的贡献区间
        // 最终贡献上限控制在 0.4 (40分)
        let sparse_contribution = (sparse_score * 20.0).tanh() * 0.4;

        // 总分 = 语义权重(20%) + 命中奖励(40%) + 词法强度(40%)
        dense_sim * 0.2 + base_boost + sparse_contribution
    };

    // 4. 映射到 0-100 整数，并进行硬性封顶
    (hybrid_score * 100.0).round().clamp(0.0, 100.0) as usize
}
