use super::*;
use acorn_lib::schema::{
    Contact, Graphics, HighlightSections, Metadata, Notes, OrganizationSections, Other, ProjectSections, Research, ResearchActivity, Sections,
};
use acorn_lib::util::{cli, MimeType};
use fast_qr::convert::image::ImageError;
use mock_it::Mock;
use percy_dom::prelude::*;
use std::collections::HashMap;

// Mock for rust-embed structures
struct MockEmbed {
    data: HashMap<String, Vec<u8>>,
}

impl MockEmbed {
    fn new() -> Self {
        MockEmbed {
            data: HashMap::new(),
        }
    }

    fn add(&mut self, key: &str, value: Vec<u8>) {
        self.data.insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<MockFile> {
        self.data.get(key).map(|data| MockFile {
            data: data.clone(),
        })
    }
}

struct MockFile {
    data: Vec<u8>,
}

// Test suite for DataUri conversions
#[cfg(test)]
mod data_uri_tests {
    use super::*;

    #[test]
    fn test_from_bytes_png() {
        let data = vec![0, 1, 2, 3];
        let result = DataUri::from_bytes(&data, MimeType::Png);
        assert!(result.starts_with("data:image/png;base64,"));
        assert_eq!(result, "data:image/png;base64,AAECAw==");
    }

    #[test]
    fn test_from_bytes_jpeg() {
        let data = vec![0, 1, 2, 3];
        let result = DataUri::from_bytes(&data, MimeType::Jpeg);
        assert!(result.starts_with("data:image/jpeg;base64,"));
        assert_eq!(result, "data:image/jpeg;base64,AAECAw==");
    }

    #[test]
    fn test_from_bytes_svg() {
        let data = vec![0, 1, 2, 3];
        let result = DataUri::from_bytes(&data, MimeType::Svg);
        assert!(result.starts_with("data:image/svg+xml;base64,"));
        assert_eq!(result, "data:image/svg+xml;base64,AAECAw==");
    }

    #[test]
    fn test_from_bytes_otf() {
        let data = vec![0, 1, 2, 3];
        let result = DataUri::from_bytes(&data, MimeType::Otf);
        assert!(result.starts_with("data:font/otf;base64,"));
        assert_eq!(result, "data:font/otf;base64,AAECAw==");
    }

    // Mock the from_asset method for testing
    #[test]
    fn test_from_asset_mocked() {
        // This would normally require modifying the implementation to accept a mock
        // For demonstration, we'll show the test structure
        // In a real implementation, we'd inject the dependencies
        
        let test_png_data = vec![1, 2, 3, 4];
        let expected = format!("data:image/png;base64,{}", BASE64.encode(&test_png_data));
        
        // Conceptual mock - actual implementation would need dependency injection
        // let mock_image = MockEmbed::new();
        // mock_image.add("test.png", test_png_data);
        // let result = DataUri::from_asset_with_mock("test.png", &mock_image);
        // assert_eq!(result, expected);
    }
}

// Test suite for HtmlHead rendering
#[cfg(test)]
mod html_head_tests {
    use super::*;
    use std::str::FromStr;

    // Helper to convert render result to string for assertion
    fn render_to_string(node: VirtualNode) -> String {
        node.to_string()
    }

    #[test]
    fn test_html_head_fact_sheet() {
        // Mock the stylesheet
        // In a real test we'd inject this dependency
        let head = HtmlHead {
            size: cli::Size::Standard,
            stylesheet: "project.css".to_string(),
            target: cli::Target::FactSheet,
        };
        
        let rendered = render_to_string(head.render());
        assert!(rendered.contains("<head>"));
        assert!(rendered.contains("<meta charset=\"utf-8\">"));
        assert!(rendered.contains("<title>Research Activity Data</title>"));
        assert!(rendered.contains("letter portrait"));
    }

    #[test]
    fn test_html_head_highlight_standard() {
        let head = HtmlHead {
            size: cli::Size::Standard,
            stylesheet: "highlight.css".to_string(),
            target: cli::Target::Highlight,
        };
        
        let rendered = render_to_string(head.render());
        assert!(rendered.contains("<head>"));
        assert!(rendered.contains("size: 10in 7.5in"));
    }

    #[test]
    fn test_html_head_highlight_widescreen() {
        let head = HtmlHead {
            size: cli::Size::Widescreen,
            stylesheet: "highlight.css".to_string(),
            target: cli::Target::Highlight,
        };
        
        let rendered = render_to_string(head.render());
        assert!(rendered.contains("<head>"));
        assert!(rendered.contains("size: 13.33in 7.5in"));
    }

    #[test]
    fn test_html_head_poster() {
        let head = HtmlHead {
            size: cli::Size::Standard,
            stylesheet: "poster.css".to_string(),
            target: cli::Target::Poster,
        };
        
        let rendered = render_to_string(head.render());
        assert!(rendered.contains("<head>"));
        // Poster shouldn't have page CSS
        assert!(!rendered.contains("@page"));
    }
}

// Test suite for HtmlFooter rendering
#[cfg(test)]
mod html_footer_tests {
    use super::*;

    fn render_to_string(node: VirtualNode) -> String {
        node.to_string()
    }

    #[test]
    fn test_html_footer_fact_sheet() {
        let footer = HtmlFooter {
            organization: "Test Organization".to_string(),
            target: cli::Target::FactSheet,
        };
        
        let rendered = render_to_string(footer.render());
        assert!(rendered.contains("<footer"));
        assert!(rendered.contains("Test Organization"));
        assert!(rendered.contains("Oak Ridge National Laboratory is managed by"));
    }

    #[test]
    fn test_html_footer_highlight() {
        let footer = HtmlFooter {
            organization: "Test Organization".to_string(),
            target: cli::Target::Highlight,
        };
        
        let rendered = render_to_string(footer.render());
        assert!(rendered.contains("<footer>"));
        assert!(rendered.contains("<div class=\"logo-wrapper\">"));
        // Highlight footer shouldn't contain the organization
        assert!(!rendered.contains("Test Organization"));
    }

    #[test]
    fn test_html_footer_poster() {
        let footer = HtmlFooter {
            organization: "Test Organization".to_string(),
            target: cli::Target::Poster,
        };
        
        let rendered = render_to_string(footer.render());
        assert!(rendered.contains("<footer></footer>"));
        // Poster footer should be empty
        assert!(!rendered.contains("Test Organization"));
    }
}

// Test suite for QR code generation
#[cfg(test)]
mod qr_code_tests {
    use super::*;

    #[test]
    fn test_generate_qr_code() {
        let url = "https://example.com/test";
        let result = generate_qr_code(url);
        
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // QR code PNG should have a minimum size
        assert!(bytes.len() > 100);
        
        // Verify it's a valid PNG by checking the magic bytes
        assert_eq!(bytes[0], 0x89);
        assert_eq!(bytes[1], 0x50);
        assert_eq!(bytes[2], 0x4E);
        assert_eq!(bytes[3], 0x47);
    }

    #[test]
    fn test_generate_qr_code_empty() {
        let url = "";
        let result = generate_qr_code(url);
        
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // Even empty QR codes should generate a valid PNG
        assert!(bytes.len() > 100);
    }

    #[test]
    #[should_panic(expected = "Failed to generate QR code")]
    fn test_generate_qr_code_too_large() {
        // Creating a string that's too large for QR code
        let huge_url = "a".repeat(2953); // Max capacity is around 2953 alphanumeric chars
        generate_qr_code(&huge_url).unwrap();
    }
}

// Test suite for stylesheet loading and processing
#[cfg(test)]
mod stylesheet_tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // Setup code that will be called once per test module
            // In a real implementation, we'd set up mocks here
        });
    }

    #[test]
    #[should_panic(expected = "unimplemented")]
    fn test_get_stylesheet_not_found() {
        setup();
        // This should panic because the asset doesn't exist
        get_stylesheet("non_existent.css".to_string());
    }

    // Mock test for get_stylesheet
    // In a real implementation, we'd inject the dependencies
    #[test]
    fn test_get_stylesheet_mocked() {
        // Conceptual mock - actual implementation would need dependency injection
        /*
        let mut mock_style = MockEmbed::new();
        mock_style.add("test.css", b"body { color: red; }".to_vec());
        
        let result = get_stylesheet_with_mock("test.css".to_string(), &mock_style);
        assert_eq!(result, "body { color: red; }");
        */
    }
}

// Test suite for font face string generation
#[cfg(test)]
mod font_face_tests {
    use super::*;

    #[test]
    fn test_generate_font_faces() {
        let result = generate_font_faces();
        
        // Check that all required font faces are present
        assert!(result.contains("@font-face {font-family:\"myriad_pro_light\";"));
        assert!(result.contains("@font-face {font-family:\"myriad_pro_semibold\";"));
        assert!(result.contains("@font-face {font-family:\"proxima\";"));
        assert!(result.contains("@font-face {font-family:\"proxima_nova_bold\";"));
        assert!(result.contains("@font-face {font-family:\"proxima_nova_semibold\";"));
        
        // Check that data URIs are included
        assert!(result.contains("data:font/otf;base64,"));
    }
}

// Test suite for complete HTML generation for each section type
#[cfg(test)]
mod html_generation_tests {
    use super::*;

    fn create_test_metadata() -> Metadata {
        Metadata {
            id: "test-id".to_string(),
            doi: Some("10.1234/test".to_string()),
            created: "2023-01-01".to_string(),
            updated: Some("2023-01-02".to_string()),
            program: Some("Test Program".to_string()),
            subprogram: Some("Test Subprogram".to_string()),
            policy: None,
            publication: Some("Test Publication".to_string()),
            graphics: Some(vec![
                Graphics {
                    href: "test.png".to_string(),
                    caption: "Test Caption".to_string(),
                    title: Some("Test Image".to_string()),
                }
            ]),
            tags: None,
        }
    }

    fn create_test_contact() -> Contact {
        Contact {
            first: "John".to_string(),
            last: "Doe".to_string(),
            username: None,
            email: "john.doe@example.com".to_string(),
            phone: None,
            role: "Principal Investigator".to_string(),
            orcid: None,
            organization: "Test Organization".to_string(),
        }
    }

    #[test]
    fn test_get_html_project() {
        let research_activity = ResearchActivity {
            meta: create_test_metadata(),
            title: "Test Project".to_string(),
            subtitle: Some("Project Subtitle".to_string()),
            contact: create_test_contact(),
            notes: None,
            sections: Sections::Project(
                ProjectSections {
                    introduction: "Test introduction".to_string(),
                    challenge: "Test challenge".to_string(),
                    approach: "Test approach".to_string(),
                    outcomes: vec!["Outcome 1".to_string(), "Outcome 2".to_string()],
                    research: Research {
                        focus: "Test focus".to_string(),
                        areas: vec!["Area 1".to_string(), "Area 2".to_string()],
                        facilities: None,
                    },
                }
            ),
        };
        
        let html = get_html(&research_activity, Some(cli::Target::FactSheet), Some(cli::Size::Standard));
        
        // Verify basic structure
        assert!(html.contains("<html>"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        
        // Verify content
        assert!(html.contains("Test Project"));
        assert!(html.contains("Project Subtitle"));
        assert!(html.contains("Test introduction"));
        assert!(html.contains("Challenge"));
        assert!(html.contains("Test challenge"));
        assert!(html.contains("Approach"));
        assert!(html.contains("Test approach"));
        assert!(html.contains("Outcomes"));
        assert!(html.contains("Outcome 1"));
        assert!(html.contains("Outcome 2"));
        assert!(html.contains("Research Focus"));
        assert!(html.contains("Test focus"));
        assert!(html.contains("Area 1"));
        assert!(html.contains("Area 2"));
    }

    #[test]
    fn test_get_html_highlight() {
        let research_activity = ResearchActivity {
            meta: create_test_metadata(),
            title: "Test Highlight".to_string(),
            subtitle: Some("Highlight Subtitle".to_string()),
            contact: create_test_contact(),
            notes: Some(Other::Formatted(Notes {
                managers: Some(vec!["Manager 1".to_string()]),
                programs: Some(vec!["Program 1".to_string()]),
                facilities: None,

