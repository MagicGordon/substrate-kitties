#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*, 
        traits::{Randomness, Currency, ReservableCurrency, ExistenceRequirement}
    };
    use frame_system::pallet_prelude::*;
    use codec::{Encode, Decode};
    use sp_io::hashing::blake2_128;

    use sp_runtime::{
        traits::{
            AtLeast32BitUnsigned, Bounded
        }
    };

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Encode, Decode, Debug)]
    pub struct Kitty(pub [u8; 16]);

    // type KittyIndex = u32;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type ValueBase: Get<BalanceOf<Self>>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittiesCreate(T::AccountId, T::KittyIndex),
        KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
        BuyKitty(T::AccountId, T::AccountId, BalanceOf<T>, T::KittyIndex),
        SellKitty(T::AccountId, T::AccountId, BalanceOf<T>, T::KittyIndex),
    }

    #[pallet::storage]
	#[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;
    
    #[pallet::storage]
	#[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;
    
    #[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

    #[pallet::error]
    pub enum Error<T> {
        KittiesCountOverflow,
        NotOwner,
        SameParentIndex,
        InvalidKittyIndex,
        ReserveFailed,
        BuyerIsOwner,
    }

    #[pallet::call]
    impl<T:Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            let dna = Self::random_value(&who);

            T::Currency::reserve(&who, T::ValueBase::get()).map_err(|_| Error::<T>::ReserveFailed)?;
            Self::create_kitty(&who, &dna)?;

            Ok(())
        }
        
        #[pallet::weight(0)]
        pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

            Owner::<T>::insert(kitty_id, Some(new_owner.clone()));

            Self::deposit_event(Event::KittyTransfer(who, new_owner, kitty_id));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);
            let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
            let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

            let dna_1 = kitty1.0;
            let dna_2 = kitty2.0;

            let selector = Self::random_value(&who);
            let mut new_dna = [0u8; 16];

            for i in 0..dna_1.len() {
                new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
            }

            Self::create_kitty(&who, &new_dna)?;

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex, value: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let _kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            let to = Owner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            ensure!(who != to, Error::<T>::BuyerIsOwner);

            Self::change_currency_owner(&to, &who, kitty_id, value)?;

            Self::deposit_event(Event::BuyKitty(to, who, value, kitty_id));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn sell(origin: OriginFor<T>, kitty_id: T::KittyIndex, buyer: T::AccountId, value: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let _ = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            let to = Owner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
            ensure!(who == to, Error::<T>::NotOwner);

            Self::change_currency_owner(&who, &buyer, kitty_id, value)?;

            Self::deposit_event(Event::SellKitty(who, buyer, value, kitty_id));
            Ok(())
        }
    }

    impl<T:Config> Pallet<T> {

        fn random_value(sender: &T::AccountId) -> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index()
            );

            payload.using_encoded(blake2_128)
        }

        fn create_kitty(who: &T::AccountId, dna: &[u8; 16]) -> Result<(), Error<T>> {
            let kitty_id = match Self::kitties_count(){
                Some(id) => {
                    ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
                    id
                },
                None => {
                    1u32.into()
                }
            };

            Kitties::<T>::insert(kitty_id, Some(Kitty(*dna)));

            Owner::<T>::insert(kitty_id, Some(who.clone()));

            KittiesCount::<T>::put(kitty_id + 1u32.into());

            Self::deposit_event(Event::KittiesCreate(who.clone(), kitty_id));

            Ok(())
        }

        fn change_currency_owner(from: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex, value: BalanceOf<T>) -> Result<(), DispatchError> {
            Owner::<T>::insert(kitty_id, Some(to.clone()));
            T::Currency::transfer(&to, &from, value, ExistenceRequirement::AllowDeath)?;
            T::Currency::unreserve(&from, T::ValueBase::get());
            Ok(())
        }
    }
}