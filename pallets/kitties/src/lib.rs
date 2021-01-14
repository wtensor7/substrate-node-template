#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Encode, Decode};
use frame_support::{
    decl_module,decl_storage, decl_event, 
    decl_error, StorageValue, ensure, StorageMap, 
    traits::Randomness,
    traits::{Get, Currency, ReservableCurrency}
};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::{DispatchError};


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// 定义一个 kitty 的数据结构
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);
// kitty 的key是u32类型
type KittyIndex = u32;
// balance
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;

    // 创建 Kitty 质押代币
	type KittyReserve: Get<BalanceOf<Self>>;
	// 质押代币操作
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}

// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
        // kitty被创建的事件
        Created(AccountId, KittyIndex),
		Transferred(AccountId, AccountId, KittyIndex),
    }
}



// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        // kitty 数量溢出
        KittiesCountOverflow,
        // kitty 不存在
        KittyNotExists,
        // 非kitty所有者
        KittyNotOwner,
        // 繁殖需要不同的父母
        RequiredDiffrentParent,
        // DOT不够
        MoneyNotEnough,
        // 不能自己转让给自己
        TransferSelf,
    }
}


// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    trait Store for Module<T: Trait> as Kitties {        
        // 所有的猫 index => kitty
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) KittyIndex => Option<Kitty>;
        //猫的总数
        pub KittiesCount get(fn kitties_count): KittyIndex;
        // 猫属于哪个所有者
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) KittyIndex => Option<T::AccountId>;
        // 记录用户与猫
        pub OwnedKitties get(fn owned_kitties):double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) KittyIndex => Option<KittyIndex>;
        // 记录父母
        pub KittyParents get(fn kitty_parents):map hasher(blake2_128_concat) KittyIndex => Option<(KittyIndex, KittyIndex)>;

        // 记录孩子关系
        pub KittyChildren get(fn kitty_children):double_map hasher(blake2_128_concat) KittyIndex, hasher(blake2_128_concat) KittyIndex => Option<KittyIndex>;

        // 记录夫妻关系
        pub KittyPartners get(fn kitty_partners):double_map hasher(blake2_128_concat) KittyIndex, hasher(blake2_128_concat) KittyIndex => Option<KittyIndex>;
    }
}


// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // 触发错误信息，必须包含这一行
		type Error = Error<T>;
		// 触发事件，必须包含这一行
        fn deposit_event() = default;

        // 随机创建一个kitty
        #[weight = 0]
        pub fn create(origin){
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_dna(&sender);
            let kitty = Kitty(dna);

            // 质押资产，防止其无限创建kitty
            T::Currency::reserve(&sender, T::KittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough)?;
            // 插入kitty, 直接创建的 无父母
            Self::insert_kitty(&sender, kitty_id, kitty, None);
			Self::deposit_event(RawEvent::Created(sender, kitty_id));

        }

        #[weight = 0]
        pub fn transfer(origin, dest: T::AccountId, kitty_id: KittyIndex){
            let sender = ensure_signed(origin)?;
            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::KittyNotExists)?;

            // 非所有者
            ensure!(sender == owner, Error::<T>::KittyNotOwner);
            // 不能转让给自己
            ensure!(sender != dest, Error::<T>::TransferSelf);
            

            // 质押代币
            T::Currency::reserve(&dest, T::KittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough )?;
            
            // 转让需要解除质押 发送者的代币
            T::Currency::unreserve(&sender, T::KittyReserve::get());

            // 修改所有人
            KittyOwners::<T>::insert(kitty_id, &dest);
            // 删除 own => kitty 关系, 插入新关系
            OwnedKitties::<T>::remove(&sender, kitty_id);
            OwnedKitties::<T>::insert(&dest, kitty_id, kitty_id);
            
            Self::deposit_event(RawEvent::Transferred(sender, dest, kitty_id));
        }

        #[weight = 0]
        pub fn breed(origin, kitty_id1: KittyIndex, kitty_id2: KittyIndex){
            let sender = ensure_signed(origin)?;
			let new_kitty_id = Self::do_breed(&sender, kitty_id1, kitty_id2)?;
			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        }

    }     
}


impl<T: Trait> Module<T> {
    fn do_breed(owner : &T::AccountId, kitty_id1: KittyIndex, kitty_id2: KittyIndex) -> sp_std::result::Result<KittyIndex, DispatchError>{
		// 同一只kitty不能繁殖
		ensure!( kitty_id1 != kitty_id2, Error::<T>::RequiredDiffrentParent);

		// 判断kitty 是否存在
		let owner1 = Self::kitty_owner(kitty_id1).ok_or( Error::<T>::KittyNotExists)?;
		let owner2 = Self::kitty_owner(kitty_id2).ok_or( Error::<T>::KittyNotExists)?;
		// 判断 两只kitty 是否都是自己的
		ensure!(owner1 == *owner, Error::<T>::KittyNotOwner);
		ensure!(owner2 == *owner, Error::<T>::KittyNotOwner);

		let kitty_1 = Self::kitties(kitty_id1).ok_or( Error::<T>::KittyNotExists )?;
		let kitty_2 = Self::kitties(kitty_id2).ok_or( Error::<T>::KittyNotExists )?;

		let kitty_id = Self::next_kitty_id()?;

		let kitty1_dna = kitty_1.0;
		let kitty2_dna = kitty_2.0;
		let selector = Self::random_dna(&owner);

		let mut new_dna = [0u8; 16];

		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		let kitty = Kitty(new_dna);

        // 质押代币
		T::Currency::reserve(&owner, T::KittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough )?;

        // 插入kitty， 父母
		Self::insert_kitty(owner, kitty_id, kitty, Some((kitty_id1, kitty_id2)));

		Ok(kitty_id)
	}

    // 获取下一个kitty的id 也就是kitties_count + 1 做溢出判断
    fn next_kitty_id() -> sp_std::result::Result<KittyIndex, DispatchError>{
		let kitty_id = Self::kitties_count();
		if kitty_id == KittyIndex::max_value() {
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
    }
    
    // 随机dna
    fn random_dna(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
			T::Randomness::random_seed(),	// 通过最近区块信息生成的随机数种子
			&sender,
			<frame_system::Module<T>>::extrinsic_index() // 当前交易在区块中的顺序
		);
		payload.using_encoded(blake2_128)
    }
    // 插入kitty和父母，有的kitty可能没有父母
    fn insert_kitty(owner : &T::AccountId, kitty_id : KittyIndex, kitty : Kitty, parent: Option<(KittyIndex, KittyIndex)> ){
		// 保存 Kitty 
		Kitties::insert(kitty_id, kitty);
		// 更新 Kitty 数量、
		KittiesCount::put(kitty_id+1);
		<KittyOwners::<T>>::insert(kitty_id, owner);
		// 保存拥有者拥有的 Kitty 数据
		<OwnedKitties::<T>>::insert(owner, kitty_id, kitty_id);
		// 保存 Kitty 的父母相关的数据
		match parent {
			Some((parent_id1, parent_id2)) =>{
				// 保存孩子 => 父母
				KittyParents::insert(kitty_id, (parent_id1, parent_id2) );
				// 保存父母 => 孩子 doublemap
				KittyChildren::insert(parent_id1, kitty_id, kitty_id);
				KittyChildren::insert(parent_id2, kitty_id, kitty_id);
				// 保存父 => 母 doublemap
				KittyPartners::insert(parent_id1, parent_id2, parent_id2);
				KittyPartners::insert(parent_id2, parent_id1, parent_id1);
			}
			_ => (),
		}
	}

}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8{
	(selector & dna1 ) | (!selector & dna2)
}