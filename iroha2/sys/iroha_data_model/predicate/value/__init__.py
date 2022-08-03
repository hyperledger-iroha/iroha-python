from ....rust import Enum, Struct, Tuple, Dict
AtIndex = Struct[("index", int), ("predicate", "iroha_data_model.predicate.value.Predicate")]

Container = Enum[("Any", "iroha_data_model.predicate.value.Predicate"), ("All", "iroha_data_model.predicate.value.Predicate"), ("AtIndex", "iroha_data_model.predicate.value.AtIndex"), ("ValueOfKey", "iroha_data_model.predicate.value.ValueOfKey"), ("HasKey", "iroha_data_model.name.Name")] 
Predicate = Enum[("Identifiable", "iroha_data_model.predicate.string.Predicate"), ("Container", "iroha_data_model.predicate.value.Container"), ("Display", "iroha_data_model.predicate.string.Predicate"), ("Numerical", "iroha_data_model.predicate.numerical.Range"), ("TimeStamp", "iroha_data_model.predicate.numerical.SemiInterval"), ("Pass", type(None))] 
ValueOfKey = Struct[("key", "iroha_data_model.name.Name"), ("predicate", "iroha_data_model.predicate.value.Predicate")]
