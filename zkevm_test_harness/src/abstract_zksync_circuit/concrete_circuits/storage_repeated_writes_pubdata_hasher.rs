use derivative::*;

use super::*;

use sync_vm::glue::traits::GenericHasher;
use sync_vm::rescue_poseidon::RescueParams;

#[derive(Derivative, serde::Serialize, serde::Deserialize)]
#[derivative(Clone, Copy, Debug, Default(bound = ""))]
pub struct StorageRepeatedWritesRehasherInstanceSynthesisFunction;

use sync_vm::glue::pubdata_hasher::hash_pubdata_entry_point;
use sync_vm::glue::pubdata_hasher::input::PubdataHasherInstanceWitness;
use sync_vm::glue::pubdata_hasher::storage_write_data::RepeatedStorageWriteData;
use sync_vm::glue::pubdata_hasher::storage_write_data::REPEATED_STORAGE_WRITE_ENCODING_LENGTH;
use sync_vm::glue::pubdata_hasher::variable_length::hash_pubdata_entry_point_variable_length;

impl<E: Engine> ZkSyncUniformSynthesisFunction<E>
    for StorageRepeatedWritesRehasherInstanceSynthesisFunction
{
    type Witness = PubdataHasherInstanceWitness<
        E,
        REPEATED_STORAGE_WRITE_ENCODING_LENGTH,
        40,
        RepeatedStorageWriteData<E>,
    >;
    type Config = usize;
    type RoundFunction = GenericHasher<E, RescueParams<E, 2, 3>, 2, 3>;

    fn description() -> String {
        "Repeated writes pubdata hasher".to_string()
    }

    fn get_synthesis_function_dyn<'a, CS: ConstraintSystem<E> + 'a>() -> Box<
        dyn FnOnce(
                &mut CS,
                Option<Self::Witness>,
                &Self::RoundFunction,
                Self::Config,
            ) -> Result<AllocatedNum<E>, SynthesisError>
            + 'a,
    > {
        // Box::new(hash_pubdata_entry_point::<_, _, _, REPEATED_STORAGE_WRITE_ENCODING_LENGTH, 40, RepeatedStorageWriteData<E>>)
        Box::new(
            hash_pubdata_entry_point_variable_length::<
                _,
                _,
                _,
                REPEATED_STORAGE_WRITE_ENCODING_LENGTH,
                40,
                RepeatedStorageWriteData<E>,
            >,
        )
    }
}
