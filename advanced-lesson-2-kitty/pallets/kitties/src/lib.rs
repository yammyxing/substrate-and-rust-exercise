#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_error, ensure, StorageValue, StorageMap, traits::Randomness, Parameter};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{AtLeast32Bit, Bounded, Member},
    DispatchError,
    DispatchResult
};

// type KittyIndex = u32;
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct KittyLinkedItem<T: Trait> {
    pub prev: Option<T::KittyIndex>,
    pub next: Option<T::KittyIndex>,
}

pub trait Trait: frame_system::Trait {
    // type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    type KittyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

decl_storage! {
    trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        pub KittyOwners get(fn kitties_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
        pub OwnedKitties get(fn owned_kitties): map hasher(blake2_128_concat) (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
	}
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        KittiesCountOverflow,
        InvalidKittyId,
        RequireDifferentParent,
        NoPermissionToTransferKitty
	}
}

// decl_event! {
    // pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        // Created(AccountId, T::KittyIndex),
        // Transferred(AccountId, AccountId, T::KittyIndex),
	// }
// }

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        // fn deposit_event() = default;

        #[weight = 0]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);
            Self::insert_kitty(&sender, kitty_id, kitty);
            // Self::deposit_event(RawEvent::Created(sender, kitty_id));
        }

        #[weight = 0]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            Self::do_transfer(&sender, &to, kitty_id)?;
            // <KittyOwners<T>>::insert(kitty_id, to.clone());
            // Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
        }

        #[weight = 0]
        pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
            // let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
            // Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        }
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> OwnedKitties<T> {
    fn read_head(account: &T::AccountId) -> KittyLinkedItem<T> {
        Self::read(account, None)
    }

    fn write_head(account: &T::AccountId, item: KittyLinkedItem<T>) {
        Self::write(account, None, item);
    }

    fn read(account: &T::AccountId, key: Option<T::KittyIndex>) -> KittyLinkedItem<T> {
        <OwnedKitties<T>>::get((&account, key)).unwrap_or_else(|| KittyLinkedItem {
            prev: None,
            next: None,
        })
    }

    fn write(account: &T::AccountId, key: Option<T::KittyIndex>, item: KittyLinkedItem<T>) {
        <OwnedKitties<T>>::insert((&account, key), item);
    }

    pub fn append(account: &T::AccountId, kitty_id: T::KittyIndex) {
        let head = Self::read_head(account);
        let new_head = KittyLinkedItem {
            prev: Some(kitty_id),
            next: head.next,
        };

        Self::write_head(account, new_head);

        let prev = Self::read(account, head.prev);
        let new_prev = KittyLinkedItem {
            prev: prev.prev,
            next: Some(kitty_id),
        };
        Self::write(account, head.prev, new_prev);

        let item = KittyLinkedItem {
            prev: head.prev,
            next: None,
        };
        Self::write(account, Some(kitty_id), item);
    }

    pub fn remove(account: &T::AccountId, kitty_id: T::KittyIndex) {
        // take = get + remove
        if let Some(item) = <OwnedKitties<T>>::take((&account, Some(kitty_id))) {
            let prev = Self::read(account, item.prev);
            let new_prev = KittyLinkedItem {
                prev: prev.prev,
                next: item.next,
            };

            Self::write(account, item.prev, new_prev);

            let next = Self::read(account, item.next);
            let new_next = KittyLinkedItem {
                prev: item.prev,
                next: next.next,
            };

            Self::write(account, item.next, new_next);
        }
    }
}

impl<T: Trait> Module<T> {
    // type Error = Error<T>;
    // fn deposit_event() = default;
    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        Kitties::<T>::insert(kitty_id, kitty);
        KittiesCount::<T>::put(kitty_id);
        <KittyOwners<T>>::insert(kitty_id, owner);
    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}

    // fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    //     (selector & dna) | (!selector & dna2);
    // }

    fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
         let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
         let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

         ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

         let kitty_id = Self::next_kitty_id()?;

         let kitty1_dna = kitty1.0;
         let kitty2_dna = kitty2.0;
         let selector = Self::random_value(&sender);
         let mut new_dna = [0u8; 16];

         for i in 0..kitty1_dna.len() {
             new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
         }

         Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

         Ok(kitty_id)
    }

    fn do_transfer(sender: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
        // 判断小猫归属，如果属于sender，才可以进行转移
        let item = <OwnedKitties<T>>::get((&sender, Some(kitty_id)));
        match item {
            Some(_v) => {
                OwnedKitties::<T>::remove(sender, kitty_id);
                OwnedKitties::<T>::append(to, kitty_id);
            }
            None => return Err(Error::<T>::NoPermissionToTransferKitty.into()),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sp_core::H256;
	use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
	};
	use frame_system as system;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq, Debug)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
        type BaseCallFilter = ();
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type DbWeight = ();
		type BlockExecutionWeight = ();
		type ExtrinsicBaseWeight = ();
		type MaximumExtrinsicWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
        type SystemWeightInfo = ();
        type PalletInfo = ();
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
    }

    type Randomness = pallet_randomness_collective_flip::Module<Test>;

	impl Trait for Test {
        // type Event = ();
        type Randomness = Randomness;
        type KittyIndex = u32;
    }

    pub type Kitties = Module<Test>;
    // pub type System = frame_system::Module<Test>;

    // fn run_to_block(n: u64) {
    //     while System::block_number() < n {
    //         Kitties::on_finalize(System::block_number());
    //         System::on_finalize(System::block_number());
    //         System::set_block_number(System::block_number() + 1);
    //         System::on_initialize(System::block_number());
    //         Kitties::on_initialize(System::block_number());
    //     }
    // }

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> sp_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn owned_kitties_can_append_values() {
		new_test_ext().execute_with(|| {
            // run_to_block(10);
			assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
		});
	}

	// #[test]
	// fn owned_kitties_can_remove_values() {
	// 	new_test_ext().execute_with(|| {

	// 	});
	// }
}