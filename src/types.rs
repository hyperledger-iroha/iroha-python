//! Module with python dictionary which can be hashed

use std::collections::btree_map::IntoIter;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

use pyo3::basic::CompareOp;
use pyo3::class::iter::{IterNextOutput, PyIterProtocol};
use pyo3::conversion::{ToBorrowedObject, ToPyObject};
use pyo3::exceptions::PyKeyError;
use pyo3::types::{PyDict, PyList};
use pyo3::{prelude::*, PyMappingProtocol, PyNativeType, PyObjectProtocol};
use pythonize::types::*;

pub use dict::Dict;
pub use list::List;

type Hash = isize;

pub mod list {
    use pyo3::PySequenceProtocol;

    use crate::to_py_err;

    use super::*;

    /// List which can be hashed. Requires all items to be hashable
    #[pyclass]
    #[derive(Debug, Clone, Default)]
    pub struct List {
        vec: Vec<PyObject>,
    }

    impl From<&PyList> for List {
        fn from(list: &PyList) -> Self {
            let py = list.py();
            let vec = list
                .into_iter()
                .map(move |i| i.to_object(py))
                .collect::<Vec<_>>();
            Self { vec }
        }
    }

    /// List which implements pythonize sequence trait
    #[derive(Clone)]
    pub struct PythonizeList<'py> {
        py: Python<'py>,
        list: List,
    }

    impl<'py> From<PythonizeList<'py>> for PyObject {
        fn from(PythonizeList { py, list }: PythonizeList<'py>) -> Self {
            list.into_py(py)
        }
    }

    impl<'py> PythonizeSequence<'py> for PythonizeList<'py> {
        fn new<T, U>(py: Python<'py>, elements: impl IntoIterator<Item = T, IntoIter = U>) -> Self
        where
            T: ToPyObject,
            U: ExactSizeIterator<Item = T>,
        {
            let vec = elements.into_iter().map(move |k| k.to_object(py)).collect();
            let list = List { vec };
            Self { py, list }
        }

        fn py(&self) -> Python<'py> {
            self.py
        }

        fn len(&self) -> usize {
            self.list.vec.len()
        }

        fn downcast(obj: &'py PyAny) -> PyResult<Self> {
            let py = obj.py();
            let list = if obj.is_instance::<PyList>()? {
                obj.downcast::<PyList>()?.into()
            } else {
                obj.extract()?
            };
            Ok(Self { py, list })
        }

        fn is_instance(obj: &'py PyAny) -> PyResult<bool> {
            Ok(obj.is_instance::<List>()? || obj.is_instance::<PyList>()?)
        }
    }

    impl List {
        pub fn hash(&self, py: Python) -> Hash {
            let sum_hash =
                |a: isize, b: isize| a.wrapping_add(b).into_py(py).as_ref(py).hash().unwrap();

            self.iter(py)
                .filter_map(|i| i.extract::<isize>().ok())
                .fold(0, sum_hash)
        }

        pub fn iter<'py>(&'py self, py: Python<'py>) -> impl Iterator<Item = &'py PyAny> + 'py {
            self.vec.iter().map(move |i| i.as_ref(py))
        }
    }

    #[pyproto]
    impl PyObjectProtocol for List {
        /// Comparison which relies on hashes
        fn __richcmp__(&self, other: Self, op: CompareOp) -> bool {
            matches!(op, CompareOp::Eq if Python::with_gil(|py| self.hash(py) == other.hash(py)))
        }

        fn __hash__(&self) -> isize {
            Python::with_gil(|py| self.hash(py))
        }

        fn __str__(&self) -> PyResult<String> {
            Python::with_gil(|py| {
                let items = self
                    .iter(py)
                    .map(|k| k.str().map(|k| k.to_string()))
                    .collect::<PyResult<Vec<_>>>()?
                    .join(",");
                Ok(format!("[{}]", items))
            })
        }

        fn __repr__(&self) -> PyResult<String> {
            self.__str__()
        }
    }

    #[pyproto]
    impl PySequenceProtocol for List {
        fn __len__(&'p self) -> usize {
            self.vec.len()
        }

        fn __getitem__(&'p self, idx: isize) -> PyResult<PyObject> {
            self.vec
                .get(idx as usize)
                .map(Clone::clone)
                .ok_or_else(|| iroha_error::error!("Failed to get item at index {}", idx))
                .map_err(to_py_err)
        }

        fn __setitem__(&mut self, idx: isize, value: PyObject) -> PyResult<()> {
            *self
                .vec
                .get_mut(idx as usize)
                .ok_or_else(|| iroha_error::error!("Failed to get item at index {}", idx))
                .map_err(to_py_err)? = value;
            Ok(())
        }
    }
}

pub mod dict {
    use super::*;

    /// Dictionary which can be hashed. Requires both keys and values to be hashable
    #[pyclass]
    #[derive(Debug, Clone, Default)]
    pub struct Dict {
        map: BTreeMap<
            // Key hash
            Hash,
            BTreeMap<
                // Value hash
                Hash,
                // Entry
                (PyObject, PyObject),
            >,
        >,
    }

    impl TryFrom<&PyDict> for Dict {
        type Error = PyErr;
        fn try_from(dict: &PyDict) -> PyResult<Self> {
            let mut map = Dict::new();
            for (k, v) in dict {
                map.set_item(dict.py(), k, v)?;
            }
            Ok(map)
        }
    }

    fn as_dict_obj(py: Python, obj: PyObject) -> PyResult<PyObject> {
        let obj = obj.into_ref(py);
        let obj = if obj.is_instance::<PyDict>()? {
            Dict::try_from(obj.downcast::<PyDict>()?)?.into_py(py)
        } else {
            obj.to_object(py)
        };
        Ok(obj)
    }

    impl Dict {
        fn is_eq(py: Python, a: &PyObject, b: &PyObject) -> bool {
            a.as_ref(py)
                .rich_compare(b.as_ref(py), CompareOp::Eq)
                .and_then(|b| b.extract::<bool>())
                .unwrap_or(false)
        }

        /// Sets item with key and value
        pub fn set_item<K: ToPyObject, V: ToPyObject>(
            &mut self,
            py: Python,
            k: K,
            v: V,
        ) -> PyResult<()> {
            let k = as_dict_obj(py, k.to_object(py))?;
            let v = as_dict_obj(py, v.to_object(py))?;
            let k_hash = k.as_ref(py).hash()?;
            let v_hash = v.as_ref(py).hash()?;
            self.map.entry(k_hash).or_default().insert(v_hash, (k, v));
            Ok(())
        }

        /// Gets item by key
        pub fn get_item<K: ToBorrowedObject>(
            &self,
            py: Python,
            k: K,
        ) -> PyResult<Option<PyObject>> {
            let k = as_dict_obj(py, k.to_object(py))?;
            let k_hash = k.as_ref(py).hash()?;
            let bucket = match self.map.get(&k_hash) {
                Some(bucket) => bucket,
                None => return Ok(None),
            };
            Ok(bucket
                .values()
                .find(|(bucket_k, _)| Self::is_eq(py, &k, bucket_k))
                .map(|(_, v)| v.clone()))
        }

        /// Remove item by key
        pub fn remove_item<K: ToBorrowedObject>(
            &mut self,
            py: Python,
            k: K,
        ) -> PyResult<Option<PyObject>> {
            let k = as_dict_obj(py, k.to_object(py))?;
            let k_hash = k.as_ref(py).hash()?;
            let bucket = match self.map.get_mut(&k_hash) {
                Some(bucket) => bucket,
                None => return Ok(None),
            };
            Ok(bucket
                .values_mut()
                .find(|(bucket_k, _)| Self::is_eq(py, &k, bucket_k))
                .map(|(_, v)| v.clone()))
        }

        /// Iterator over both keys and values
        pub fn iter<'py>(
            &'py self,
            py: Python<'py>,
        ) -> impl Iterator<Item = (&'py PyAny, &'py PyAny)> + 'py {
            self.map
                .values()
                .flat_map(BTreeMap::values)
                .map(move |(k, v)| (k.clone().into_ref(py), v.clone().into_ref(py)))
        }

        /// Iterator which consumes object
        pub fn into_iter(self, py: Python<'_>) -> impl Iterator<Item = (&'_ PyAny, &'_ PyAny)> {
            self.map
                .into_iter()
                .flat_map(|(_, bucket)| bucket.into_iter())
                .map(move |(_, (k, v))| (k.into_ref(py), v.into_ref(py)))
        }

        /// Iterator over keys
        pub fn keys<'py>(&'py self, py: Python<'py>) -> impl Iterator<Item = &'py PyAny> + 'py {
            self.iter(py).map(|(k, _)| k)
        }

        /// Iterator over keys which consumes object
        pub fn into_keys(self, py: Python<'_>) -> impl Iterator<Item = &'_ PyAny> {
            self.into_iter(py).map(|(k, _)| k)
        }

        /// Iterator over values
        pub fn values<'py>(&'py self, py: Python<'py>) -> impl Iterator<Item = &'py PyAny> + 'py {
            self.iter(py).map(|(_, v)| v)
        }

        /// Iterator over values which consumes object
        pub fn into_values(self, py: Python<'_>) -> impl Iterator<Item = &'_ PyAny> {
            self.into_iter(py).map(|(_, v)| v)
        }

        /// Hashes object
        pub fn hash(&self, py: Python) -> Hash {
            let sum_hash =
                |a: isize, b: isize| a.wrapping_add(b).into_py(py).as_ref(py).hash().unwrap();

            self.map
                .iter()
                .flat_map(|(&key, bucket)| bucket.keys().map(move |&value| sum_hash(key, value)))
                .fold(0, sum_hash)
        }

        /// Returns number of items in dictionary
        pub fn len(&self) -> usize {
            self.map.iter().fold(0, |prev, (_, next)| prev + next.len())
        }

        /// Checks whether dict is empty
        pub fn is_empty(&self) -> bool {
            self.map.len() == 0
        }
    }

    #[pymethods]
    impl Dict {
        /// Constructor
        #[new]
        pub fn new() -> Self {
            Self::default()
        }

        /// Returns an iterator over keys
        #[pyo3(name = "keys")]
        fn _keys(&self) -> Keys {
            Keys::new(self.clone())
        }

        /// Returns an iterator over values
        #[pyo3(name = "values")]
        fn _values(&self) -> Values {
            Values::new(self.clone())
        }

        /// Returns an iterator over both keys and values
        fn items(&self) -> Items {
            Items::new(self.clone())
        }
    }

    #[pyproto]
    impl PyObjectProtocol for Dict {
        /// Comparison which relies on hashes
        fn __richcmp__(&self, other: Self, op: CompareOp) -> bool {
            matches!(op, CompareOp::Eq if Python::with_gil(|py| self.hash(py) == other.hash(py)))
        }

        fn __hash__(&self) -> isize {
            Python::with_gil(|py| self.hash(py))
        }

        fn __str__(&self) -> PyResult<String> {
            Python::with_gil(|py| {
                let items = self
                    .iter(py)
                    .map(|(k, v)| Ok(format!("{}: {}", k.str()?, v.str()?)))
                    .collect::<PyResult<Vec<_>>>()?
                    .join(",");
                Ok(format!("{{{}}}", items))
            })
        }

        fn __repr__(&self) -> PyResult<String> {
            Python::with_gil(|py| {
                let items = self
                    .iter(py)
                    .map(|(k, v)| Ok(format!("{}: {}", k.repr()?, v.repr()?)))
                    .collect::<PyResult<Vec<_>>>()?
                    .join(",");
                Ok(format!("{{{}}}", items))
            })
        }
    }

    #[pyproto]
    impl PyMappingProtocol for Dict {
        fn __len__(&self) -> usize {
            self.len()
        }

        fn __setitem__(&mut self, key: PyObject, value: PyObject) -> PyResult<()> {
            Python::with_gil(|py| self.set_item(py, key, value))
        }

        fn __delitem__(&mut self, key: PyObject) -> PyResult<()> {
            Python::with_gil(|py| self.remove_item(py, key))?;
            Ok(())
        }

        fn __getitem__(&self, key: PyObject) -> PyResult<PyObject> {
            match Python::with_gil(|py| self.get_item(py, key.clone()))? {
                Some(obj) => Ok(obj),
                None => Err(PyErr::new::<PyKeyError, _>(key)),
            }
        }
    }

    #[pyclass]
    struct Items {
        dict: IntoIter<Hash, BTreeMap<Hash, (PyObject, PyObject)>>,
        bucket: Option<IntoIter<Hash, (PyObject, PyObject)>>,
    }

    impl Items {
        fn new(dict: Dict) -> Self {
            Self {
                dict: dict.map.into_iter(),
                bucket: None,
            }
        }

        fn next(&mut self) -> Option<(PyObject, PyObject)> {
            if let Some(bucket) = &mut self.bucket {
                if let Some((_, entry)) = bucket.next() {
                    return Some(entry);
                }
            }
            self.bucket = match self.dict.next() {
                Some((_, bucket)) => Some(bucket.into_iter()),
                None => return None,
            };

            self.next()
        }
    }

    #[pyclass]
    struct Keys {
        items: Items,
    }

    impl Keys {
        fn new(dict: Dict) -> Self {
            let items = Items::new(dict);
            Self { items }
        }
    }

    #[pyclass]
    struct Values {
        items: Items,
    }

    impl Values {
        fn new(dict: Dict) -> Self {
            let items = Items::new(dict);
            Self { items }
        }
    }

    #[pyproto]
    impl PyIterProtocol for Items {
        fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<(PyObject, PyObject), ()> {
            match slf.next() {
                Some(entry) => IterNextOutput::Yield(entry),
                None => IterNextOutput::Return(()),
            }
        }
    }

    #[pyproto]
    impl PyIterProtocol for Keys {
        fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<PyObject, ()> {
            match slf.items.next() {
                Some((entry, _)) => IterNextOutput::Yield(entry),
                None => IterNextOutput::Return(()),
            }
        }
    }

    #[pyproto]
    impl PyIterProtocol for Values {
        fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<PyObject, ()> {
            match slf.items.next() {
                Some((_, entry)) => IterNextOutput::Yield(entry),
                None => IterNextOutput::Return(()),
            }
        }
    }

    /// Pythonize dictionary which implements pythonize mapping trait
    #[derive(Clone)]
    pub struct PythonizeDict<'py> {
        py: Python<'py>,
        dict: Dict,
    }

    impl<'py> From<PythonizeDict<'py>> for Py<PyAny> {
        fn from(PythonizeDict { py, dict }: PythonizeDict<'py>) -> Self {
            dict.into_py(py)
        }
    }

    impl<'py> PythonizeMapping<'py> for PythonizeDict<'py> {
        type PyIter = std::vec::IntoIter<(PyObject, PyObject)>;

        fn new(py: Python<'py>) -> Self {
            Self {
                py,
                dict: Default::default(),
            }
        }

        fn py(&self) -> Python<'py> {
            self.py
        }

        fn set_item<V, K>(&mut self, key: &K, value: &V) -> PyResult<()>
        where
            K: ToPyObject,
            V: ToPyObject,
        {
            self.dict.set_item(self.py, key, value)
        }

        fn get_item<K>(&self, key: &K) -> Option<&PyAny>
        where
            K: ToBorrowedObject,
        {
            self.dict
                .get_item(self.py, key)
                .ok()
                .flatten()
                .map(|v| v.into_ref(self.py))
        }

        fn len(&self) -> usize {
            self.dict.len()
        }

        fn keys(&self) -> &PyList {
            PyList::new(self.py, self.dict.keys(self.py).collect::<Vec<_>>())
        }

        fn iter(&self) -> Self::PyIter {
            self.dict
                .iter(self.py)
                .map(|(k, v)| (k.into_py(self.py), v.into_py(self.py)))
                .collect::<Vec<_>>()
                .into_iter()
        }

        fn downcast(obj: &'py PyAny) -> PyResult<Self> {
            let py = obj.py();
            let dict = if obj.is_instance::<PyDict>()? {
                obj.downcast::<PyDict>()?.try_into()?
            } else {
                obj.extract()?
            };
            Ok(Self { py, dict })
        }

        fn is_instance(obj: &PyAny) -> PyResult<bool> {
            Ok(obj.is_instance::<Dict>()? || obj.is_instance::<PyDict>()?)
        }
    }
}
