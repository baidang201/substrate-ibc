//! Benchmarking setup for pallet-template

// todo need to fix
// benchmarks::benchmarking::bench_ack_packet_mock
// benchmarks::benchmarking::bench_channel_close_confirm_mock
// benchmarks::benchmarking::bench_channel_close_init_mock
// benchmarks::benchmarking::bench_channel_open_ack_mock
// benchmarks::benchmarking::bench_channel_open_confirm_mock
// benchmarks::benchmarking::bench_channel_open_try_mock
// benchmarks::benchmarking::bench_conn_open_ack_mock
// benchmarks::benchmarking::bench_conn_open_confirm_mock
// benchmarks::benchmarking::bench_conn_try_open_mock
// benchmarks::benchmarking::bench_create_client_mock
// benchmarks::benchmarking::bench_recv_packet_mock
// benchmarks::benchmarking::bench_timeout_packet_mock
// benchmarks::benchmarking::bench_upgrade_mock_client
#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use ibc_proto::google::protobuf::Any;

use ibc::mock::client_state as mock_client_state;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

use alloc::boxed::Box;
use ibc::core::{
	ics02_client::{
		client_state::ClientState,
		height::Height,
		msgs::{
			create_client::{MsgCreateClient, TYPE_URL as CREATE_CLIENT_TYPE_URL},
			update_client::TYPE_URL as UPDATE_CLIENT_TYPE_URL,
		},
	},
	ics03_connection::{
		connection::{ConnectionEnd, Counterparty, State},
		msgs::{
			conn_open_ack::TYPE_URL as CONN_OPEN_ACK_TYPE_URL,
			conn_open_confirm::TYPE_URL as CONN_OPEN_CONFIRM_TYPE_URL,
			conn_open_init as conn_open_init_mod,
			conn_open_try::TYPE_URL as CONN_TRY_OPEN_TYPE_URL,
		},
		version::Version as ConnVersion,
	},
	ics04_channel::{
		channel::ChannelEnd,
		msgs::{
			acknowledgement::TYPE_URL as ACK_PACKET_TYPE_URL,
			chan_close_confirm::TYPE_URL as CHAN_CLOSE_CONFIRM_TYPE_URL,
			chan_close_init::TYPE_URL as CHAN_CLOSE_INIT_TYPE_URL,
			chan_open_ack::TYPE_URL as CHAN_OPEN_ACK_TYPE_URL,
			chan_open_confirm::TYPE_URL as CHAN_OPEN_CONFIRM_TYPE_URL,
			chan_open_init::{MsgChannelOpenInit, TYPE_URL as CHAN_OPEN_TYPE_URL},
			chan_open_try::TYPE_URL as CHAN_OPEN_TRY_TYPE_URL,
			recv_packet::TYPE_URL as RECV_PACKET_TYPE_URL,
			timeout::TYPE_URL as TIMEOUT_TYPE_URL,
		},
	},
	ics23_commitment::commitment::CommitmentPrefix,
	ics24_host::{
		identifier::{ChannelId, ClientId, ConnectionId, PortId},
		path::{
			ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
			ClientTypePath, ConnectionPath, SeqRecvPath, SeqSendPath,
		},
	},
	ics26_routing::msgs::MsgEnvelope,
	ExecutionContext, ValidationContext,
};
use ibc_proto::protobuf::Protobuf;
use scale_info::prelude::string::ToString;
use sp_std::vec;

use super::utils::TIMESTAMP;

benchmarks! {
	where_clause {
		where u32: From<<T as frame_system::Config>::BlockNumber>,
				<T as frame_system::Config>::BlockNumber: From<u32>,
			T: Send + Sync + pallet_timestamp::Config<Moment = u64>
	}

	// Run these benchmarks via
	// ```bash
	// cargo +nightly test -p pallet-ibc  --features=runtime-benchmarks
	// ```
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);

	create_client_mock {
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let msg = MsgCreateClient::new(
			mock_client_state.into(),
			mock_cs_state.into(),
			crate::tests::common::get_dummy_account_id(),
		).encode_vec().unwrap();

		let msg = Any { type_url: CREATE_CLIENT_TYPE_URL.to_string(), value: msg };
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {
	}

	// update_client
	update_mock_client {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let height = Height::new(0, 1).unwrap();
		let (mock_cl_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_cl_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);

		let msg = Any { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	upgrade_mock_client {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let height = Height::new(0, 1).unwrap();
		let (mock_cl_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_cl_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_upgrade_client(client_id.clone(), new_height);

		let msg = Any { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// misbehaviours client
	conn_open_init_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();

		let value = conn_open_init_mod::MsgConnectionOpenInit {
			client_id_on_a: client_id.clone(),
			counterparty: Counterparty::new(
				counterparty_client_id.clone(),
				Some(ConnectionId::new(1)),
				commitment_prefix.clone(),
			),
			version: Some(ConnVersion::default()),
			delay_period: core::time::Duration::from_secs(1000),
			signer: crate::tests::common::get_dummy_account_id(),
		}.encode_vec().unwrap();

		let msg = Any {
			type_url: conn_open_init_mod::TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // connection open try
	conn_try_open_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_cl_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_cl_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();


		// We update the light client state so it can have the required client and consensus states required to process
		// the proofs that will be submitted
		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let (cs_state, value) = super::utils::create_conn_open_try::<T>(new_height, Height::new(0, 3).unwrap());
		// Update consensus state with the new root that we'll enable proofs to be correctly verified
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let msg = Any { type_url: CONN_TRY_OPEN_TYPE_URL.to_string(), value };
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // connection open ack
	conn_open_ack_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		// Create a connection end and put in storage
		// Successful processing of a connection open confirm message requires a compatible connection end with state INIT or TRYOPEN
		// to exist on the local chain
		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Init, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let (cs_state, value) = super::utils::create_conn_open_ack::<T>(new_height, Height::new(0, 3).unwrap());
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let msg = Any { type_url: CONN_OPEN_ACK_TYPE_URL.to_string(), value };
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // connection open confirm
	conn_open_confirm_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		// Create a connection end and put in storage
		// Successful processing of a connection open confirm message requires a compatible connection end with state TryOpen
		// to exist on the local chain
		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::TryOpen, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		// We update the light client state so it can have the required client and consensus states required to process
		// the proofs that will be submitted
		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let (cs_state, value) = super::utils::create_conn_open_confirm(new_height);
		// Update consensus state with the new root that we'll enable proofs to be correctly verified
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let msg = Any { type_url: CONN_OPEN_CONFIRM_TYPE_URL.to_string(), value };
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {
	}


	// For all channel messages to be processed successfully, a connection end must exist and be in the OPEN state
	// create channel
	channel_open_init_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let port_id = PortId::default();
		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), None);
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Init,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let value = MsgChannelOpenInit {
			port_id_on_a: port_id.clone(),
			connection_hops_on_a: vec![ConnectionId::new(0)],
			port_id_on_b: port_id,
			ordering: ibc::core::ics04_channel::channel::Order::Ordered,
			signer: crate::tests::common::get_dummy_account_id(),
			version_proposal: ibc::core::ics04_channel::Version::default()
		}.encode_vec().unwrap();

		let caller: T::AccountId = whitelisted_caller();
		let msg = Any { type_url: CHAN_OPEN_TYPE_URL.to_string(), value };
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {
	}

	// channel_open_try
	channel_open_try_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state.client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		// We update the light client state so it can have the required client and consensus states required to process
		// the proofs that will be submitted
		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let (cs_state, value) = super::utils::create_chan_open_try(new_height);

		// Update consensus root for light client
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();

		let msg = Any {
			type_url: CHAN_OPEN_TRY_TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {
	}

	// channel_open_ack
	channel_open_ack_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state.client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let port_id = PortId::transfer();

		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::default()));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Init,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let value = MsgChannelOpenInit {
			port_id_on_a: port_id.clone(),
			connection_hops_on_a: vec![ConnectionId::new(0)],
			port_id_on_b: port_id,
			ordering: ibc::core::ics04_channel::channel::Order::Ordered,
			signer: crate::tests::common::get_dummy_account_id(),
			version_proposal: ibc::core::ics04_channel::Version::default()
		}.encode_vec().unwrap();

		let msg = ibc_proto::google::protobuf::Any  { type_url: CHAN_OPEN_TYPE_URL.to_string(), value };

		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let (cs_state, value) = super::utils::create_chan_open_ack(new_height);

		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: CHAN_OPEN_ACK_TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // channel_open_confirm
	channel_open_confirm_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let port_id = PortId::transfer();

		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::TryOpen,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let channel_end_path = ChannelEndPath(port_id.clone(), ChannelId::new(0));
		ctx.store_channel(&channel_end_path, channel_end).unwrap();

		let (cs_state, value) = super::utils::create_chan_open_confirm(new_height);
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 2,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: CHAN_OPEN_CONFIRM_TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // channel_close_init
	channel_close_init_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let port_id = PortId::transfer();

		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let channel_end_path = ChannelEndPath(port_id.clone(), ChannelId::new(0));
		ctx.store_channel(&channel_end_path, channel_end).unwrap();

		let (_, value) = super::utils::create_chan_close_init(new_height);

		let msg = Any {
			type_url: CHAN_CLOSE_INIT_TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// // channel_close_confirm
	channel_close_confirm_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(1000);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);
		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let port_id = PortId::transfer();

		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let channel_end_path = ChannelEndPath(port_id.clone(), ChannelId::new(0));
		ctx.store_channel(&channel_end_path, channel_end).unwrap();

		let (cs_state, value) = super::utils::create_chan_close_confirm(new_height);
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 2,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: CHAN_CLOSE_CONFIRM_TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	// recv_packet
	recv_packet_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(0);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let port_id = PortId::transfer();
		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Unordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let channel_end_path = ChannelEndPath(port_id.clone(), ChannelId::new(0));
		ctx.store_channel(&channel_end_path, channel_end).unwrap();
		let seq_rev_path = SeqRecvPath(port_id.clone(), ChannelId::new(0));
		ctx.store_next_sequence_recv(&seq_rev_path, 1u64.into()).unwrap();

		let (cs_state, value) = super::utils::create_recv_packet(new_height);
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 2,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: RECV_PACKET_TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {
	}

	// ack_packet
	ack_packet_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();

		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(0);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);

		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();
		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };

		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let port_id = PortId::transfer();
		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Unordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let channel_end_path = ChannelEndPath(port_id.clone(), ChannelId::new(0));
		ctx.store_channel(&channel_end_path, channel_end).unwrap();
		let seq_rev_path = SeqRecvPath(port_id.clone(), ChannelId::new(0));
		ctx.store_next_sequence_recv(&seq_rev_path, 1u64.into()).unwrap();

		let (cs_state, value) = super::utils::create_ack_packet(new_height);
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 2,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: ACK_PACKET_TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}

	timeout_packet_mock {
		let mut ctx = crate::context::Context::<T>::new();
		// Set timestamp to the same timestamp used in generating tendermint header, because there
		// will be a comparison between the local timestamp and the timestamp existing in the header
		// after factoring in the trusting period for the light client.
		let now: <T as pallet_timestamp::Config>::Moment = TIMESTAMP.saturating_mul(1000);
		pallet_timestamp::Pallet::<T>::set_timestamp(now);
		let number : <T as frame_system::Config>::BlockNumber = 1u32.into();
		frame_system::Pallet::<T>::set_block_number(number);
		let height = Height::new(0, 1).unwrap();
		let (mock_client_state, mock_cs_state) = super::utils::create_mock_state(height);
		let client_id = ClientId::new(mock_client_state::client_type(), 0).unwrap();
		let counterparty_client_id = ClientId::new(mock_client_state::client_type(), 1).unwrap();
		let client_type_path = ClientTypePath(client_id.clone());
		ctx.store_client_type(client_type_path, mock_client_state::client_type()).unwrap();
		let client_state_path = ClientStatePath(client_id.clone());
		ctx.store_client_state(client_state_path, Box::new(mock_client_state)).unwrap();
		let consensus_state_path = ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: 0,
			height: 1,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(mock_cs_state)).unwrap();
		let connection_id = ConnectionId::new(0);
		let commitment_prefix: CommitmentPrefix = "ibc".as_bytes().to_vec().try_into().unwrap();
		let delay_period = core::time::Duration::from_nanos(0);
		let connection_counterparty = Counterparty::new(counterparty_client_id, Some(ConnectionId::new(1)), commitment_prefix);
		let connection_end = ConnectionEnd::new(State::Open, client_id.clone(), connection_counterparty, vec![ConnVersion::default()], delay_period);
		let connection_path = ConnectionPath(connection_id.clone());
		ctx.store_connection(&connection_path, connection_end).unwrap();

		let client_connection_path = ClientConnectionPath(client_id.clone());
		ctx.store_connection_to_client(&client_connection_path, connection_id.clone()).unwrap();

		let new_height = Height::new(0, 2).unwrap();
		let value = super::utils::create_mock_update_client(client_id.clone(), new_height);

		let msg = ibc_proto::google::protobuf::Any  { type_url: UPDATE_CLIENT_TYPE_URL.to_string(), value };
		let envelope: MsgEnvelope = msg.try_into().unwrap();
		ibc::core::handler::dispatch(&mut ctx, envelope).unwrap();

		let port_id = PortId::transfer();
		let counterparty_channel = ibc::core::ics04_channel::channel::Counterparty::new(port_id.clone(), Some(ChannelId::new(0)));
		let channel_end = ChannelEnd::new(
			ibc::core::ics04_channel::channel::State::Open,
			ibc::core::ics04_channel::channel::Order::Ordered,
			counterparty_channel,
			vec![ConnectionId::new(0)],
			ibc::core::ics04_channel::Version::default()
		);

		let channel_end_path = ChannelEndPath(port_id.clone(), ChannelId::new(0));
		ctx.store_channel(&channel_end_path, channel_end).unwrap();

		let seq_rev_path = SeqRecvPath(port_id.clone(), ChannelId::new(0));
		ctx.store_next_sequence_recv(&seq_rev_path, 1u64.into()).unwrap();

		let seq_send_path = SeqSendPath(port_id.clone(), ChannelId::new(0));
		ctx.store_next_sequence_send(&seq_send_path, 1u64.into()).unwrap();

		let (cs_state, value) = super::utils::create_timeout_packet(new_height);
		let consensus_state_path = ClientConsensusStatePath {
			client_id,
			epoch:0,
			height:2,
		};
		ctx.store_consensus_state(consensus_state_path, Box::new(cs_state)).unwrap();
		let msg = Any {
			type_url: TIMEOUT_TYPE_URL.to_string(),
			value
		};
		let caller: T::AccountId = whitelisted_caller();
	}: dispatch(RawOrigin::Signed(caller), vec![msg])
	verify {

	}
}
