#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
    traits::Randomness, RuntimeDebug, StorageDoubleMap, StorageValue,
}; // StorageValue and StorageDoubleMap is needed if we want to access the get method in our module
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum KittyGender {
    Male,
    Female,
}
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Kitty(pub [u8; 16]); // Our type Kitty

impl Kitty {
    pub fn gender(&self) -> KittyGender {
        if self.0[0] % 2 == 0 {
            KittyGender::Male
        } else {
            KittyGender::Female
        }
    }
}

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event! {
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
    {
        /// A kitty is created. \[owner, kitty_id, kitty\]
        KittyCreated(AccountId, u32, Kitty),
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Kitties {
        /// Stores all the kitties, key is the kitty id
        pub Kitties get(fn kitties): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u32 => Option<Kitty>;
        /// Stores the next kitty ID
        pub NextKittyId get(fn next_kitty_id): u32;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        KittiesIdOverflow,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 1000]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;

            NextKittyId::try_mutate(|next_id| -> DispatchResult {
                let current_id = *next_id;
                *next_id = next_id.checked_add(1).ok_or(Error::<T>::KittiesIdOverflow)?;

                // Generate a random 128bit value
                let payload = (
                    <pallet_randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
                    &sender,
                    <frame_system::Module<T>>::extrinsic_index(),
                );
                let dna = payload.using_encoded(blake2_128);

                // Create and store kitty
                let kitty = Kitty(dna);
                Kitties::<T>::insert(&sender, current_id, kitty.clone());

                // Emit event
                Self::deposit_event(RawEvent::KittyCreated(sender, current_id, kitty));

                Ok(())
            })?;
        }

    }
}
