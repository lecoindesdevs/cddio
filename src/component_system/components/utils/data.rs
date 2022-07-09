//! Module de gestion des données des composants.
//! 

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::{fs, env};

use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Serialize};

lazy_static! {
    /// Chemin du dossier contenant les données.
    pub static ref DATA_DIR: PathBuf = env::current_dir().unwrap().join("data");
}
#[derive(Debug)]
pub enum DataError {
    /// Erreur lors de la lecture/écriture du fichier.
    FileError(std::io::Error),
    /// Erreur de sérialisation/déserialisation.
    SerdeError(ron::error::Error),
    /// Le fichier n'existe pas dans le dossier [`DATA_DIR`].
    MissingFileError,
}
use DataError::*;

pub type DataResult<T> = std::result::Result<T, DataError>;

/// Gestionnaire de donnée.
/// 
/// La structure contient les données d'un composant. Elle se charge de la lecture et de l'enregistrement des données sur le disque dur.
pub struct Data<T> 
    where T: DeserializeOwned + Serialize 
{
    pub name: String,
    pub value: T,
}
impl<T> Debug for Data<T> 
    where T: DeserializeOwned + Serialize + Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Data {{ name: {}, value: {:?} }}", self.name, self.value)
    }
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
    pub fn from_file<S: AsRef<str>>(name: S) -> DataResult<Data<T>> {
        let path_data = DATA_DIR.join(format!("{}.ron", name.as_ref()));
        if !path_data.exists() {
            return Err(DataError::MissingFileError);
        }
        let file = fs::File::open(path_data).or_else(|e| Err(FileError(e)))?;
        let data = Data { 
            name: name.as_ref().to_string(), 
            value: ron::de::from_reader(file).or_else(|e| Err(SerdeError(e)))?
        };
        
        Ok(data)
    }
    /// Charge une donnée depuis un fichier. 
    /// 
    /// Si le fichier n'existe pas, le paramètre `default` est utilisé pour initialiser la donnée.
    /// 
    /// Préférez la fonction [`from_file_default`](Data<T>::from_file_default()) si votre Le type de données implémente [`Default`].
    /// 
    pub fn from_file_or<S: AsRef<str>>(name: S, default: T) -> DataResult<Data<T>> {
        match Self::from_file(name.as_ref()) {
            Err(DataError::MissingFileError) => Ok(Data::new(name.as_ref(), default)),
            v => v
        }
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
    /// Si le fichier n'existe pas, une nouvelle donnée est créée avec [`Default::default()`].
    pub fn from_file_default<S: AsRef<str>>(name: S) -> DataResult<Data<T>> {
        match Self::from_file(name.as_ref()) {
            Err(DataError::MissingFileError) => Ok(Data::new(name.as_ref(), T::default())),
            v => v
        }
    }
}

/// Gère l'enregistrement des données d'un composant.
/// 
/// Dès que le [`DataGuard`] est détruit, les données sont enregistrées dans le fichier correspondant.
pub struct DataGuard<'a, T>(&'a mut Data<T>)
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
        if !DATA_DIR.exists() {
            if let Err(e) = fs::create_dir_all(DATA_DIR.as_path()) {
                eprintln!("Saving {} - Unable to create the data directory: {}", self.0.name, e);
                return;
            }
        }
        let path_file = DATA_DIR.join(format!("{}.ron", self.0.name));

        fs::write(path_file, &ron_content).unwrap_or_else(|err| {
            eprintln!("Saving {} - Unable to write the file: {}", self.0.name, err);
        });
    }
}