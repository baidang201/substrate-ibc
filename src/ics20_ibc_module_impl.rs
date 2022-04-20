use super::*;
use crate::ics20_handler;
use core::str::FromStr;
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as Ics20Error,
		msgs::fungible_token_packet_data::FungibleTokenPacketData,
	},
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			msgs::acknowledgement_response::Acknowledgement,
			packet::Packet,
			Version,
		},
		ics05_port::capabilities::Capability,
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::ibc_module::IBCModule,
	},
	signer::Signer,
};
use prost::Message;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ics20IBCModule<T: Config>(core::marker::PhantomData<T>);

impl<T: Config> Ics20IBCModule<T> {
	pub fn new() -> Self {
		Self(core::marker::PhantomData)
	}
}

impl<T: Config> IBCModule for Ics20IBCModule<T> {
	// OnChanOpenInit implements the IBCModule interface
	// refter to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L64
	fn on_chan_open_init<Ctx>(
		&self,
		ctx: &Ctx,
		order: Order,
		connection_hops: Vec<ConnectionId>,
		port_id: PortId,
		channel_id: ChannelId,
		channel_cap: &Capability,
		counterparty: Counterparty,
		version: Version,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		let ret = validate_transfer_channel_params(ctx, order, port_id, channel_id)?;

		if version != Version::ics20() {
			return Err(Ics20Error::invalid_version(version, Version::ics20()))
		}

		// todo
		// Claim channel capability passed back by IBC module
		//if err := im.keeper.ClaimCapability(ctx, chanCap, host.ChannelCapabilityPath(portID,
		// channelID)); err != nil { 	return err
		//}

		Ok(())
	}

	// OnChanOpenTry implements the IBCModule interface.
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L91
	fn on_chan_open_try<Ctx>(
		&self,
		ctx: &Ctx,
		order: Order,
		connection_hops: Vec<ConnectionId>,
		port_id: PortId,
		channel_id: ChannelId,
		channel_cap: &Capability,
		counterparty: Counterparty,
		counterparty_version: Version,
	) -> Result<Version, Ics20Error>
	where
		Ctx: Ics20Context,
	{
		let ret = validate_transfer_channel_params(ctx, order, port_id, channel_id)?;

		if counterparty_version != Version::ics20() {
			return Err(Ics20Error::invalid_version(counterparty_version, Version::ics20()))
		}

		// todo
		// Module may have already claimed capability in OnChanOpenInit in the case of crossing
		// hellos (ie chainA and chainB both call ChanOpenInit before one of them calls ChanOpenTry)
		// If module can already authenticate the capability then module already owns it so we don't
		// need to claim Otherwise, module does not have channel capability and we must claim it
		// from IBC if !im.keeper.AuthenticateCapability(ctx, chanCap,
		// host.ChannelCapabilityPath(portID, channelID)) { Only claim channel capability passed
		// back by IBC module if we do not already own it if err := im.keeper.ClaimCapability(ctx,
		// chanCap, host.ChannelCapabilityPath(portID, channelID)); err != nil { 	return "", err
		//}

		Ok(Version::ics20())
	}

	// OnChanOpenAck implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L124
	fn on_chan_open_ack<Ctx>(
		&self,
		ctx: &Ctx,
		port_id: PortId,
		channel_id: ChannelId,
		counterparty_version: Version,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		if counterparty_version != Version::ics20() {
			return Err(Ics20Error::invalid_version(counterparty_version, Version::ics20()))
		}
		Ok(())
	}

	// OnChanOpenConfirm implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L137
	fn on_chan_open_confirm<Ctx>(
		&self,
		ctx: &Ctx,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		Ok(())
	}

	// OnChanCloseInit implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L146
	fn on_chan_close_init<Ctx>(
		&self,
		ctx: &Ctx,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		// Disallow user-initiated channel closing for transfer channels
		Err(Ics20Error::invalid_request())
	}
	// OnChanCloseConfirm implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L156
	fn on_chan_close_confirm<Ctx>(
		&self,
		ctx: &Ctx,
		port_id: PortId,
		channel_id: ChannelId,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		Ok(())
	}
	// OnRecvPacket implements the IBCModule interface. A successful acknowledgement
	// is returned if the packet data is succesfully decoded and the receive application
	// logic returns without error.
	// refer to ibc-go https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L167
	fn on_recv_packet<Ctx>(
		&self,
		ctx: &Ctx,
		packet: Packet,
		relayer: Signer,
	) -> Result<Vec<u8>, Ics20Error>
	where
		Ctx: Ics20Context,
	{
		// 	ack := channeltypes.NewResultAcknowledgement([]byte{byte(1)})
		let default_ack_value = vec![1];
		// construct Acknowledgement
		let mut acknowledgement = Acknowledgement::new_success(default_ack_value);

		// build FungibleTokenPacketData
		let data = FungibleTokenPacketData::decode(&mut &packet.data[..])
			.map_err(|e| Ics20Error::invalid_decode(e))?;

		// only attempt the application logic if the packet data
		// was successfully decoded
		if acknowledgement.success().map_err(|e| Ics20Error::ics04_channel(e))? {
			// handle recv packet
			let result = ics20_handler::handle_recv_packet::<Ctx, T>(ctx, packet, data);
			if let Err(err) = result {
				acknowledgement = Acknowledgement::new_error(format!("{}", err));
			}
		}

		let ack = acknowledgement.encode_vec().map_err(|e| Ics20Error::invalid_encode(e))?;
		Ok(ack)
	}

	// OnAcknowledgementPacket implements the IBCModule interface
	// refer to ibc-go https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L204
	fn on_acknowledgement_packet<Ctx>(
		&self,
		ctx: &Ctx,
		packet: Packet,
		acknowledgement: Vec<u8>,
		relayer: Signer,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		let ack = Acknowledgement::decode(&mut &acknowledgement[..])
			.map_err(|e| Ics20Error::invalid_decode(e))?;

		let data = FungibleTokenPacketData::decode(&mut &packet.data[..])
			.map_err(|e| Ics20Error::invalid_decode(e))?;

		let ret = ics20_handler::handle_ack_packet::<Ctx, T>(ctx, packet, data, ack.into())?;

		Ok(())
	}

	// OnTimeoutPacket implements the IBCModule interface
	// refer to https://github.com/octopus-network/ibc-go/blob/ac46ac06084f586a460b092b2b293a321b7c43d6/modules/apps/transfer/ibc_module.go#L204
	fn on_timeout_packet<Ctx>(
		&self,
		ctx: &Ctx,
		packet: Packet,
		relayer: Signer,
	) -> Result<(), Ics20Error>
	where
		Ctx: Ics20Context,
	{
		let data = FungibleTokenPacketData::decode(&mut &packet.data[..])
			.map_err(|e| Ics20Error::invalid_decode(e))?;

		// handle ack packet/refund tokens
		let ret = ics20_handler::handle_timeout_packet::<Ctx, T>(ctx, packet, data)?;

		Ok(())
	}
}

// validate_transfer_channel_params does validation of a newly created transfer channel. A transfer
// channel must be UNORDERED, use the correct port (by default 'transfer'), and use the current
// supported version. Only 2^32 channels are allowed to be created.
fn validate_transfer_channel_params<Ctx: Ics20Context>(
	ctx: &Ctx,
	order: Order,
	port_id: PortId,
	channel_id: ChannelId,
) -> Result<(), Ics20Error> {
	// NOTE: for escrow address security only 2^32 channels are allowed to be created
	// Issue: https://github.com/cosmos/cosmos-sdk/issues/7737
	let channel_sequence = parse_channel_sequence(channel_id.0)?;

	if channel_sequence > u32::MAX.into() {
		return Err(Ics20Error::overflow_channel_sequence(channel_sequence, u32::MAX.into()))
	}

	if order != Order::Unordered {
		return Err(Ics20Error::invalid_equal_order(Order::Unordered, order))
	}

	// Require portID is the portID transfer module is bound to
	let bound_port = ctx.get_port()?;
	if bound_port != port_id {
		return Err(Ics20Error::invalid_equal_port_id(bound_port, port_id))
	}

	Ok(())
}

// parse_channel_sequence parses the channel sequence from the channel identifier.
fn parse_channel_sequence(channel_identifier: String) -> Result<u64, Ics20Error> {
	let channel_id =
		ibc::core::ics24_host::identifier::ChannelId::from_str(channel_identifier.as_str())
			.map_err(|e| Ics20Error::invalid_channel_id(channel_identifier, e))?;

	let sequence = channel_id.as_str().split_once("channel-").ok_or(Ics20Error::invalid_split())?.1;

	let sequence = sequence.parse::<u64>().map_err(|_| Ics20Error::invalid_parse())?;

	Ok(sequence)
}
