//! THIS IS A GENERATED FILE!
//! Take care when hand editing. Changes will be lost during subsequent runs of the code generator.
//!
//! version: 0.1.6
//!

#![allow(dead_code)]
#![allow(unused_imports)]

use yaserde_derive::{YaSerialize, YaDeserialize};
use std::io::{Read, Write};
use log::{warn, debug};

pub const SOAP_ENCODING: &str = "http://www.w3.org/2003/05/soap-encoding";

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
pub struct Header {}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Fault",
namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
prefix = "soapenv",
)]
pub struct SoapFault {
    #[yaserde(rename = "faultcode", default)]
    pub fault_code: Option<String>,
    #[yaserde(rename = "faultstring", default)]
    pub fault_string: Option<String>,
}

impl std::error::Error for SoapFault {}

impl std::fmt::Display for SoapFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.fault_code, &self.fault_string) {
            (None, None) => Ok(()),
            (None, Some(fault_string)) => f.write_str(fault_string),
            (Some(fault_code), None) => f.write_str(fault_code),
            (Some(fault_code), Some(fault_string)) => {
                f.write_str(fault_code)?;
                f.write_str(": ")?;
                f.write_str(fault_string)
            }
        }
    }
}

pub type SoapResponse = Result<(reqwest::StatusCode, String), reqwest::Error>;

pub mod messages {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;
}

pub mod types {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "elementType",
    )]
    pub struct ElementType {
        #[yaserde(rename = "type", attribute)]
        pub r#type: String,
        #[yaserde(rename = "title", default)]
        pub title: String,
        #[yaserde(rename = "url", default)]
        pub url: String,
        #[yaserde(rename = "totalNewItems", default)]
        pub total_new_items: i32,
        #[yaserde(rename = "subElement", default)]
        pub sub_element: Vec<SubelementBaseType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "subelementBaseType",
    )]
    pub struct SubelementBaseType {
        #[yaserde(rename = "lastUpdated", attribute)]
        pub last_updated: String,
        #[yaserde(rename = "type", attribute)]
        pub r#type: String,
        #[yaserde(rename = "description", default)]
        pub description: String,
        #[yaserde(rename = "tooltip", default)]
        pub tooltip: String,
        #[yaserde(rename = "title", default)]
        pub title: String,
        #[yaserde(rename = "url", default)]
        pub url: String,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "surfaceType",
    )]
    pub struct SurfaceType {
        #[yaserde(rename = "background", attribute)]
        pub background: String,
        #[yaserde(rename = "foreground", attribute)]
        pub foreground: String,
        #[yaserde(rename = "fontFace", attribute)]
        pub font_face: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "clientAreaType",
    )]
    pub struct ClientAreaType {
        #[yaserde(flatten, default)]
        pub surface_type: SurfaceType,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "borderType",
    )]
    pub struct BorderType {
        #[yaserde(rename = "topLeftImage", attribute)]
        pub top_left_image: String,
        #[yaserde(rename = "bottomLeftImage", attribute)]
        pub bottom_left_image: String,
        #[yaserde(rename = "topRightImage", attribute)]
        pub top_right_image: String,
        #[yaserde(rename = "bottomRightImage", attribute)]
        pub bottom_right_image: String,
        #[yaserde(rename = "outline", attribute)]
        pub outline: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "liveThemeType",
    )]
    pub struct LiveThemeType {
        #[yaserde(rename = "themeName", default)]
        pub theme_name: String,
        #[yaserde(rename = "head", default)]
        pub head: HeadType,
        #[yaserde(rename = "body", default)]
        pub body: BodyType,
        #[yaserde(rename = "actions", default)]
        pub actions: LinkType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "linkType",
    )]
    pub struct LinkType {
        #[yaserde(rename = "linkColor", attribute)]
        pub link_color: String,
        #[yaserde(rename = "backgroundColor", attribute)]
        pub background_color: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "linkTextType",
    )]
    pub struct LinkTextType {
        #[yaserde(flatten, default)]
        pub link_type: LinkType,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "headType",
    )]
    pub struct HeadType {
        #[yaserde(flatten, default)]
        pub link_text_type: LinkTextType,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "bodyType",
    )]
    pub struct BodyType {
        #[yaserde(flatten, default)]
        pub head_type: HeadType,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "refreshInformationType",
    )]
    pub struct RefreshInformationType {
        #[yaserde(rename = "cid", default)]
        pub cid: String,
        #[yaserde(rename = "storageAuthCache", default)]
        pub storage_auth_cache: String,
        #[yaserde(rename = "market", default)]
        pub market: String,
        #[yaserde(rename = "brand", default)]
        pub brand: String,
        #[yaserde(rename = "maxElementCount", default)]
        pub max_element_count: i32,
        #[yaserde(rename = "maxCharacterCount", default)]
        pub max_character_count: i32,
        #[yaserde(rename = "maxImageCount", default)]
        pub max_image_count: i32,
        #[yaserde(rename = "applicationId", default)]
        pub application_id: String,
        #[yaserde(rename = "updateAccessedTime", default)]
        pub update_accessed_time: bool,
        #[yaserde(rename = "spaceLastViewed", default)]
        pub space_last_viewed: Option<String>,
        #[yaserde(rename = "isActiveContact", default)]
        pub is_active_contact: bool,
        #[yaserde(rename = "profileLastViewed", default)]
        pub profile_last_viewed: Option<String>,
        #[yaserde(rename = "contactProfileLastViewed", default)]
        pub contact_profile_last_viewed: Option<String>,
        #[yaserde(rename = "activeContactLastChanged", default)]
        pub active_contact_last_changed: Option<String>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "contactCardType",
    )]
    pub struct ContactCardType {
        #[yaserde(rename = "storageAuthCache", default)]
        pub storage_auth_cache: String,
        #[yaserde(rename = "elements", default)]
        pub elements: ElementsType,
        #[yaserde(rename = "theme", default)]
        pub theme: ThemeType,
        #[yaserde(rename = "liveTheme", default)]
        pub live_theme: LiveThemeType,
        #[yaserde(rename = "lastUpdate", default)]
        pub last_update: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "themeType",
    )]
    pub struct ThemeType {
        #[yaserde(rename = "name", default)]
        pub name: String,
        #[yaserde(rename = "titleBar", default)]
        pub title_bar: SurfaceType,
        #[yaserde(rename = "clientArea", default)]
        pub client_area: ClientAreaType,
        #[yaserde(rename = "toolbar", default)]
        pub toolbar: SurfaceType,
        #[yaserde(rename = "border", default)]
        pub border: BorderType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "elementsType",
    )]
    pub struct ElementsType {
        #[yaserde(rename = "totalMatches", attribute)]
        pub total_matches: i32,
        #[yaserde(rename = "returnedMatches", attribute)]
        pub returned_matches: i32,
        #[yaserde(rename = "displayName", attribute)]
        pub display_name: String,
        #[yaserde(rename = "displayPictureUrl", attribute)]
        pub display_picture_url: String,
        #[yaserde(rename = "element", default)]
        pub element: Vec<ElementType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "spaceContactCardElementsElementPhotoSubElement",
    )]
    pub struct SpaceContactCardElementsElementPhotoSubElement {
        #[yaserde(flatten, default)]
        pub subelement_base_type: SubelementBaseType,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
        #[yaserde(rename = "thumbnailUrl", default)]
        pub thumbnail_url: String,
        #[yaserde(rename = "webReadyUrl", default)]
        pub web_ready_url: String,
        #[yaserde(rename = "albumName", default)]
        pub album_name: String,
    }
}

pub mod ports {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;
}

pub mod bindings {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;
}

pub mod services {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;
}

