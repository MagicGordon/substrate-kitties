use crate as pallet_kitties;
use sp_core::H256;
use frame_support::parameter_types;
use frame_support::traits::StorageMapShim;

use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		KittiesModule: pallet_kitties::{Pallet, Call, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl pallet_randomness_collective_flip::Config for Test {}

impl system::Config for Test {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = u64;
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	// type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	// type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const MaxLocks: u32 = 50;
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ReserveIdentifier = [u8; 8];
	type ExistentialDeposit = ExistentialDeposit;
	type MaxReserves = ();
	type AccountStore = StorageMapShim<
			pallet_balances::Account<Test>,
			system::Provider<Test>,
			u64,
			pallet_balances::AccountData<u64>,
		>;
	type WeightInfo = ();
}

parameter_types! {
	pub const ValueBase: u64 = 100 * 1_000 * 1_000_000_000;
}

impl pallet_kitties::Config for Test {
	type KittyIndex = u32;
	type Event = Event;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type ValueBase = ValueBase;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10000 * 1_000 * 1_000_000_000), (2, 10000 * 1_000 * 1_000_000_000), (3, 10000), (4, 10000 * 1_000 * 1_000_000_000)],
	}.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}