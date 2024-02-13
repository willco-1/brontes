use alloy_primitives::U256;
use brontes_macros::action_impl;
use brontes_pricing::Protocol;
use brontes_types::{
    constants::{ETH_ADDRESS, WETH_ADDRESS},
    normalized_actions::NormalizedSwap,
    structured_trace::CallInfo,
    ToScaledRational,
};

action_impl!(
    Protocol::CurveV1MetapoolImpl,
    crate::CurveV1MetapoolImpl::exchange_0Call,
    Swap,
    [..TokenExchange],
    logs: true,
    |
    info: CallInfo,
    log: CurveV1MetapoolImplexchange_0CallLogs,
    db_tx: &DB|{
        let log = log.TokenExchange_field;

        let details = db_tx.get_protocol_details(info.from_address)?;

        let token_in_addr = match log.sold_id {
            0 => details.token0,
            1 => details.token1,
            2 => details.token2.ok_or(eyre::eyre!("Expected token2 for token in, found None"))?,
            3 => details.token3.ok_or(eyre::eyre!("Expected token3 for token in, found None"))?,
            4 => details.token4.ok_or(eyre::eyre!("Expected token4 for token in, found None"))?,
            _ => unreachable!()
        };

        let token_out_addr = match log.bought_id {
            0 => details.token0,
            1 => details.token1,
            2 => details.token2.ok_or(eyre::eyre!("Expected token2 for token out, found None"))?,
            3 => details.token3.ok_or(eyre::eyre!("Expected token3 for token out, found None"))?,
            4 => details.token4.ok_or(eyre::eyre!("Expected token4 for token out, found None"))?,
            _ => unreachable!()
        };

        let token_in = db_tx.try_fetch_token_info(token_in_addr)?;
        let token_out = db_tx.try_fetch_token_info(token_out_addr)?;

        let amount_in = log.tokens_sold.to_scaled_rational(token_in.decimals);
        let amount_out = log.tokens_bought.to_scaled_rational(token_out.decimals);


        Ok(NormalizedSwap {
            protocol: details.protocol,
            trace_index: info.trace_idx,
            pool: info.from_address,
            from: info.msg_sender,
            recipient: info.msg_sender,
            token_in,
            token_out,
            amount_in,
            amount_out,
            msg_value: info.msg_value
        })
    }
);

action_impl!(
    Protocol::CurveV1MetapoolImpl,
    crate::CurveV1MetapoolImpl::exchange_underlying_0Call,
    Swap,
    [..TokenExchangeUnderlying],
    logs: true,
    |
    info: CallInfo,
    log: CurveV1MetapoolImplexchange_underlying_0CallLogs,
    db_tx: &DB|{
        let log = log.TokenExchangeUnderlying_field;

        let details = db_tx.get_protocol_details(info.from_address)?;

        let token_in_addr = match log.sold_id {
            0 => details.token0,
            1 => details.token1,
            2 => details.token2.ok_or(eyre::eyre!("Expected token2 for token in, found None"))?,
            3 => details.token3.ok_or(eyre::eyre!("Expected token3 for token in, found None"))?,
            4 => details.token4.ok_or(eyre::eyre!("Expected token4 for token in, found None"))?,
            _ => unreachable!()
        };

        let token_out_addr = match log.bought_id {
            0 => details.token0,
            1 => details.token1,
            2 => details.token2.ok_or(eyre::eyre!("Expected token2 for token out, found None"))?,
            3 => details.token3.ok_or(eyre::eyre!("Expected token3 for token out, found None"))?,
            4 => details.token4.ok_or(eyre::eyre!("Expected token4 for token out, found None"))?,
            _ => unreachable!()
        };

        let token_in = db_tx.try_fetch_token_info(token_in_addr)?;
        let token_out = db_tx.try_fetch_token_info(token_out_addr)?;

        let amount_in = log.tokens_sold.to_scaled_rational(token_in.decimals);
        let amount_out = log.tokens_bought.to_scaled_rational(token_out.decimals);


        Ok(NormalizedSwap {
            protocol: details.protocol,
            trace_index: info.trace_idx,
            pool: info.from_address,
            from: info.msg_sender,
            recipient: info.msg_sender,
            token_in,
            token_out,
            amount_in,
            amount_out,
            msg_value: info.msg_value
        })
    }
);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::{hex, Address, B256, U256};
    use brontes_classifier::test_utils::ClassifierTestUtils;
    use brontes_types::{
        db::token_info::{TokenInfo, TokenInfoWithAddress},
        normalized_actions::Actions,
        Node, ToScaledRational, TreeSearchArgs,
    };

    use super::*;

    #[brontes_macros::test]
    async fn test_curve_v1_metapool_impl_exchange0() {
        let classifier_utils = ClassifierTestUtils::new().await;
        classifier_utils.ensure_protocol(
            Protocol::CurveV1MetaPool,
            Address::new(hex!("A77d09743F77052950C4eb4e6547E9665299BecD")),
            Address::new(hex!("6967299e9F3d5312740Aa61dEe6E9ea658958e31")),
            Address::new(hex!("6c3f90f043a72fa612cbac8115ee7e52bde6e490")),
            None,
            None,
            None,
        );

        // CurveV1MetapoolImpl
        let swap =
            B256::from(hex!("0c673f1ede30f20bb7ca3e7c05a71dcc49a8bb18498e148e3967bb7173d6794e"));

        let token_in = TokenInfoWithAddress {
            address: Address::new(hex!("6967299e9F3d5312740Aa61dEe6E9ea658958e31")),
            inner:   TokenInfo { decimals: 18, symbol: "T".to_string() },
        };

        let token_out = TokenInfoWithAddress {
            address: Address::new(hex!("6c3f90f043a72fa612cbac8115ee7e52bde6e490")),
            inner:   TokenInfo { decimals: 18, symbol: "3Crv".to_string() },
        };

        classifier_utils.ensure_token(token_in.clone());
        classifier_utils.ensure_token(token_out.clone());

        let eq_action = Actions::Swap(NormalizedSwap {
            protocol: Protocol::CurveV1MetaPool,
            trace_index: 1,
            from: Address::new(hex!("41ce1Af5B4eF2E124028dea59580817898def508")),
            recipient: Address::new(hex!("41ce1Af5B4eF2E124028dea59580817898def508")),
            pool: Address::new(hex!("A77d09743F77052950C4eb4e6547E9665299BecD")),
            token_in,
            amount_in: U256::from_str("108987327295834489846250")
                .unwrap()
                .to_scaled_rational(18),
            token_out,
            amount_out: U256::from_str("846346204017353217859")
                .unwrap()
                .to_scaled_rational(18),
            msg_value: U256::ZERO,
        });

        let search_fn = |node: &Node<Actions>| TreeSearchArgs {
            collect_current_node:  node.data.is_swap(),
            child_node_to_collect: node
                .get_all_sub_actions()
                .iter()
                .any(|action| action.is_swap()),
        };

        classifier_utils
            .contains_action(swap, 0, eq_action, search_fn)
            .await
            .unwrap()
    }

    #[brontes_macros::test]
    async fn test_curve_v1_metapool_impl_exchange_underlying0() {
        let classifier_utils = ClassifierTestUtils::new().await;
        classifier_utils.ensure_protocol(
            Protocol::CurveV1MetaPool,
            Address::new(hex!("84997FAFC913f1613F51Bb0E2b5854222900514B")),
            Address::new(hex!("BE4fe13A73675c49A17f3524602634913C668B4C")),
            Address::new(hex!("6c3F90f043a72FA612cbac8115EE7e52BDe6E490")),
            None,
            None,
            None,
        );

        classifier_utils.ensure_protocol(
            Protocol::CurveBasePool,
            Address::new(hex!("bEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7")),
            Address::new(hex!("6B175474E89094C44Da98b954EedeAC495271d0F")),
            Address::new(hex!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")),
            Some(Address::new(hex!("dAC17F958D2ee523a2206206994597C13D831ec7"))),
            None,
            None,
        );

        let swap =
            B256::from(hex!("3eed7ebe18acfd9f68d34710f0e279989e41d475372f14a91d0d0a98d381375e"));

        let three_crv = TokenInfoWithAddress {
            address: Address::new(hex!("6c3F90f043a72FA612cbac8115EE7e52BDe6E490")),
            inner:   TokenInfo { decimals: 18, symbol: "3Crv".to_string() },
        };

        let token_in = TokenInfoWithAddress {
            address: Address::new(hex!("BE4fe13A73675c49A17f3524602634913C668B4C")),
            inner:   TokenInfo { decimals: 18, symbol: "A".to_string() },
        };

        let token_out = TokenInfoWithAddress {
            address: Address::new(hex!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")),
            inner:   TokenInfo { decimals: 6, symbol: "USDC".to_string() },
        };

        classifier_utils.ensure_token(token_in.clone());
        classifier_utils.ensure_token(token_out.clone());
        classifier_utils.ensure_token(three_crv.clone());

        let eq_action = Actions::Swap(NormalizedSwap {
            protocol: Protocol::CurveV1MetaPool,
            trace_index: 0,
            from: Address::new(hex!("49fab288ccF3E237088Ba8AC9628273D616a537d")),
            recipient: Address::new(hex!("49fab288ccF3E237088Ba8AC9628273D616a537d")),
            pool: Address::new(hex!("84997FAFC913f1613F51Bb0E2b5854222900514B")),
            token_in,
            amount_in: U256::from_str("1100000000000000000000")
                .unwrap()
                .to_scaled_rational(18),
            token_out,
            amount_out: U256::from_str("1292367389").unwrap().to_scaled_rational(6),
            msg_value: U256::ZERO,
        });

        let search_fn = |node: &Node<Actions>| TreeSearchArgs {
            collect_current_node:  node.data.is_swap(),
            child_node_to_collect: node
                .get_all_sub_actions()
                .iter()
                .any(|action| action.is_swap()),
        };

        classifier_utils
            .contains_action(swap, 0, eq_action, search_fn)
            .await
            .unwrap();
    }
}