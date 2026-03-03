use agones::ObjectMeta;

#[derive(Default, Clone, Debug)]
/// parsed from labels and annotations
pub struct AgonesMetadata {
    pub labels: AgonesLabels,
    pub annotations: AgonesAnnotations,
}

impl AgonesMetadata {
    pub fn has_match_composer(&self) -> bool {
        self.labels.match_composer
    }
}

impl TryFrom<ObjectMeta> for AgonesMetadata {
    type Error = String;

    fn try_from(value: ObjectMeta) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&ObjectMeta> for AgonesMetadata {
    type Error = String;

    fn try_from(value: &ObjectMeta) -> Result<Self, Self::Error> {
        let labels = AgonesLabels::try_from(value)?;
        let annotations = AgonesAnnotations::try_from(value)?;
        
        Ok(
            Self {
                labels,
                annotations,
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct AgonesLabels {
    pub match_composer: bool,
    pub match_composer_port: Option<u16>,
}

impl Default for AgonesLabels {
    fn default() -> Self {
        Self {
            match_composer: false,
            match_composer_port: None,
        }
    }
}

impl TryFrom<&ObjectMeta> for AgonesLabels {
    type Error = String;

    fn try_from(meta: &ObjectMeta) -> Result<Self, Self::Error> {
        let mut ret = Self::default();
        
        if  let Some(val) = meta.labels.get("match-composer") && 
            let Ok(match_composer) = val.parse::<bool>() {
            ret.match_composer = match_composer
        }
        
        if let Some(val) = meta.labels.get("match-composer-port") && 
            let Ok(port) = val.parse::<u16>() {
            ret.match_composer_port = Some(port);
        }
        
        Ok(ret)
    }
}

#[derive(Clone, Debug)]
pub struct AgonesAnnotations {
    
}

impl Default for AgonesAnnotations {
    fn default() -> Self {
        Self {
            
        }
    }
}

impl TryFrom<&ObjectMeta> for AgonesAnnotations {
    type Error = String;

    fn try_from(_meta: &ObjectMeta) -> Result<Self, Self::Error> {
        Ok(Self::default())
    }
}
