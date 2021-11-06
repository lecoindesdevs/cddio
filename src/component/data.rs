//! Module de gestion des données des composants.
//! 
//! La structure [`Data`] contient les données d'un composant. C'est cettte structure qui se charge de la lecture et de l'enregistrement des données.
//! 

use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::{fs, env};

use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

lazy_static! {
    /// Chemin du dossier contenant les données.
    static ref DATA_DIR: PathBuf = env::current_dir().unwrap().join("data");
}

struct Data<T> 
    where T: DeserializeOwned + Serialize 
{
    pub name: String,
    pub value: T,
}

impl<T> Data<T> 
    where T: DeserializeOwned + Serialize 
{
    /// Crée une nouvelle donnée.
    pub fn new(name: &str, value: T) -> Data<T> {
        Data {
            name: name.to_string(),
            value: value,
        }
    }
    /// Charge une donnée depuis un fichier. 
    /// 
    /// Si le fichier n'existe pas, une nouvelle donnée est créée.
    pub fn from_file<S: AsRef<str>>(name: S) -> Result<Data<T>, String> {
        let path_data = DATA_DIR.join(format!("{}.ron", name.as_ref()));
        if !path_data.exists() {
            return Err(format!("Le fichier {} n'existe pas", path_data.display()));
        }
        let file = fs::File::open(path_data).or_else(|e| Err("Loading {} - Unable to open the data file: {}".to_string()))?;
        let data = Data { 
            name: name.as_ref().to_string(), 
            value: ron::de::from_reader(file).or_else(|e| Err("Loading {} - Unable to deserialize the data file: {}".to_string()))?
        };
        
        Ok(data)
    }
    /// Accède en lecture aux données. 
    /// 
    /// Aucune lecture ni enregistrement de fichier n'est effectué.
    pub fn read(&self) -> &T {
        &self.value
    }
    /// Accède en écriture aux données.
    /// 
    /// Retourne un [`DataGuard`] qui vous permet d'écrire dans les données.
    pub fn write<'a>(&'a mut self) -> DataGuard<'a, T> {
        DataGuard(self)
    }
}
impl<T> Data<T> 
    where T: DeserializeOwned + Serialize + Default
{
    /// Charge une donnée depuis un fichier. 
    /// 
    /// Si le fichier n'existe pas, une nouvelle donnée est créée.
    pub fn from_file_default<S: AsRef<str>>(name: S) -> Result<Data<T>, String> {
        let path_data = DATA_DIR.join(format!("{}.ron", name.as_ref()));
        if !path_data.exists() {
            return Ok(Data::new(name.as_ref(), T::default()));
        }
        let file = fs::File::open(path_data).or_else(|e| Err("Loading {} - Unable to open the data file: {}".to_string()))?;
        let data = Data { 
            name: name.as_ref().to_string(), 
            value: ron::de::from_reader(file).or_else(|e| Err("Loading {} - Unable to deserialize the data file: {}".to_string()))?
        };
        
        Ok(data)
    }
}



impl<T> Default for Data<T> 
    where T: Default + DeserializeOwned + Serialize
{
    fn default() -> Self {
        Data {
            name: String::new(),
            value: Default::default(),
        }
    }
}

/// Gère l'enregistrement des données d'un composant.
/// 
/// Dès que le [`DataGuard`] est détruit, les données sont enregistrées dans le fichier correspondant.
struct DataGuard<'a, T>(&'a mut Data<T>)
    where T:Serialize + DeserializeOwned;

impl<T> Deref for DataGuard<'_, T> 
where T: DeserializeOwned + Serialize 
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}
impl<T> DerefMut for DataGuard<'_, T> 
where T: DeserializeOwned + Serialize 
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.value
    }
}

impl<T> Drop for DataGuard<'_, T> 
where T: DeserializeOwned + Serialize 
{
    fn drop(&mut self) {
        let ron_content = match ron::ser::to_string_pretty(&self.0.value, ron::ser::PrettyConfig::default()) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Saving {} - Unable to serialize the data: {}", self.0.name, err);
                return;
            }
        };
        let path_file = DATA_DIR.join(format!("{}.ron", self.0.name));

        fs::write(path_file, &ron_content).unwrap_or_else(|err| {
            eprintln!("Saving {} - Unable to write the file: {}", self.0.name, err);
        });
    }
}