use std::{fmt, fs::File, io::Write, path::Path, write};

use base64::decode;
use serde::ser::{Serialize, SerializeMap, Serializer};

use crate::common::command::MAGIC_ELEMENTID;
use crate::sync::WebDriver;
use crate::{
    common::{
        command::Command,
        connection_common::unwrap,
        keys::TypingData,
        types::{ElementId, ElementRect, ElementRef},
    },
    error::WebDriverResult,
    By,
};

/// Unwrap the raw JSON into a WebElement struct.
pub fn unwrap_element_sync(
    driver: WebDriver,
    value: &serde_json::Value,
) -> WebDriverResult<WebElement> {
    let elem_id: ElementRef = serde_json::from_value(value.clone())?;
    Ok(WebElement::new(driver, ElementId::from(elem_id.id)))
}

/// Unwrap the raw JSON into a Vec of WebElement structs.
pub fn unwrap_elements_sync(
    driver: &WebDriver,
    value: &serde_json::Value,
) -> WebDriverResult<Vec<WebElement>> {
    let values: Vec<ElementRef> = serde_json::from_value(value.clone())?;
    Ok(values
        .into_iter()
        .map(|x| WebElement::new(driver.clone_without_capabilities(), ElementId::from(x.id)))
        .collect())
}

/// The WebElement struct encapsulates a single element on a page.
///
/// WebElement structs are generally not constructed manually, but rather
/// they are returned from a 'find_element()' operation using a WebDriver.
///
/// # Example:
/// ```rust
/// # use thirtyfour::error::WebDriverResult;
/// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     let caps = DesiredCapabilities::chrome();
/// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
/// #     driver.get("http://webappdemo")?;
/// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
/// let elem = driver.find_element(By::Name("input-result"))?;
/// #     assert_eq!(elem.get_attribute("name")?, "input-result");
/// #     Ok(())
/// # }
/// ```
///
/// You can also search for a child element of another element as follows:
/// ```rust
/// # use thirtyfour::error::WebDriverResult;
/// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     let caps = DesiredCapabilities::chrome();
/// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
/// #     driver.get("http://webappdemo")?;
/// let elem = driver.find_element(By::Css("div[data-section='section-buttons']"))?;
/// let child_elem = elem.find_element(By::Tag("button"))?;
/// #     child_elem.click()?;
/// #     let result_elem = elem.find_element(By::Id("button-result"))?;
/// #     assert_eq!(result_elem.text()?, "Button 1 clicked");
/// #     Ok(())
/// # }
/// ```
///
/// Elements can be clicked using the `click()` method, and you can send
/// input to an element using the `send_keys()` method.
///
#[derive(Debug, Clone)]
pub struct WebElement {
    pub element_id: ElementId,
    driver: WebDriver,
}

impl WebElement {
    /// Create a new WebElement struct.
    ///
    /// Typically you would not call this directly. WebElement structs are
    /// usually constructed by calling one of the find_element*() methods
    /// either on WebDriver or another WebElement.
    pub fn new(driver: WebDriver, element_id: ElementId) -> Self {
        WebElement { element_id, driver }
    }

    ///Convenience wrapper for executing a WebDriver command.
    fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.driver.cmd(command)
    }

    /// Get the bounding rectangle for this WebElement.
    pub fn rect(&self) -> WebDriverResult<ElementRect> {
        let v = self.cmd(Command::GetElementRect(&self.element_id))?;
        let r: ElementRect = serde_json::from_value((&v["value"]).clone())?;
        Ok(r)
    }

    /// Get the tag name for this WebElement.
    pub fn tag_name(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementTagName(&self.element_id))?;
        unwrap(&v["value"])
    }

    /// Get the text contents for this WebElement.
    pub fn text(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementText(&self.element_id))?;
        unwrap(&v["value"])
    }

    /// Click the WebElement.
    pub fn click(&self) -> WebDriverResult<()> {
        self.cmd(Command::ElementClick(&self.element_id))?;
        Ok(())
    }

    /// Clear the WebElement contents.
    pub fn clear(&self) -> WebDriverResult<()> {
        self.cmd(Command::ElementClear(&self.element_id))?;
        Ok(())
    }

    /// Get the specified property.
    pub fn get_property(&self, name: &str) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementProperty(
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    /// Get the specified attribute.
    pub fn get_attribute(&self, name: &str) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementAttribute(
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    /// Get the specified CSS property.
    pub fn get_css_property(&self, name: &str) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementCSSValue(
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    /// Return true if the WebElement is currently selected, otherwise false.
    pub fn is_selected(&self) -> WebDriverResult<bool> {
        let v = self.cmd(Command::IsElementSelected(&self.element_id))?;
        unwrap(&v["value"])
    }

    /// Return true if the WebElement is currently enabled, otherwise false.
    pub fn is_enabled(&self) -> WebDriverResult<bool> {
        let v = self.cmd(Command::IsElementEnabled(&self.element_id))?;
        unwrap(&v["value"])
    }

    /// Search for a child element of this WebElement using the specified
    /// selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Css("div[data-section='section-buttons']"))?;
    /// let child_elem = elem.find_element(By::Tag("button"))?;
    /// #     child_elem.click()?;
    /// #     let result_elem = elem.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(result_elem.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self.cmd(Command::FindElementFromElement(&self.element_id, by))?;
        unwrap_element_sync(self.driver.clone(), &v["value"])
    }

    /// Search for all child elements of this WebElement that match the
    /// specified selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Css("div[data-section='section-buttons']"))?;
    /// let child_elems = elem.find_elements(By::Tag("button"))?;
    /// #     assert_eq!(child_elems.len(), 2);
    /// for child_elem in child_elems {
    ///     assert_eq!(child_elem.tag_name()?, "button");
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self.cmd(Command::FindElementsFromElement(&self.element_id, by))?;
        unwrap_elements_sync(&self.driver, &v["value"])
    }

    /// Send the specified input.
    ///
    /// # Example:
    /// You can specify anything that implements `Into<TypingData>`. This
    /// includes &str and String.
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     let elem = driver.find_element(By::Name("input1"))?;
    /// elem.send_keys("selenium")?;
    /// #     assert_eq!(elem.text()?, "selenium");
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// You can also send special key combinations like this:
    /// ```rust
    /// use thirtyfour::{Keys, TypingData};
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     let elem = driver.find_element(By::Name("input1"))?;
    /// elem.send_keys("selenium")?;
    /// elem.send_keys(Keys::Control + "a")?;
    /// elem.send_keys(TypingData::from("thirtyfour") + Keys::Enter)?;
    /// #     assert_eq!(elem.text()?, "thirtyfour");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn send_keys<S>(&self, keys: S) -> WebDriverResult<()>
    where
        S: Into<TypingData>,
    {
        self.cmd(Command::ElementSendKeys(&self.element_id, keys.into()))?;
        Ok(())
    }

    /// Take a screenshot of this WebElement and return it as a base64-encoded
    /// String.
    pub fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::TakeElementScreenshot(&self.element_id))?;
        unwrap(&v["value"])
    }

    /// Take a screenshot of this WebElement and return it as PNG bytes.
    pub fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64()?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of this WebElement and write it to the specified
    /// filename.
    pub fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png()?;
        let mut file = File::create(path)?;
        file.write_all(&png)?;
        Ok(())
    }
}

impl fmt::Display for WebElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"(session="{}", element="{}")"#,
            self.driver.session_id, self.element_id
        )
    }
}

impl Serialize for WebElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(MAGIC_ELEMENTID, &self.element_id.to_string())?;
        map.end()
    }
}
