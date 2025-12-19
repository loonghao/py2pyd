//! Unit tests for the build_tools module
//!
//! These tests verify build tools detection functionality.

#[cfg(test)]
mod build_tools_tests {
    /// Test that detect_build_tools doesn't panic
    #[test]
    fn test_detect_build_tools_no_panic() {
        let tools = py2pyd::detect_build_tools();

        // Just verify it returns without panicking
        let _ = tools.has_msvc();
        let _ = tools.has_mingw();
        let _ = tools.has_gcc();
        let _ = tools.has_xcode();
        let _ = tools.has_any_tools();
    }

    /// Test BuildTools info string generation
    #[test]
    fn test_build_tools_info_string() {
        let tools = py2pyd::detect_build_tools();
        let info = tools.get_tools_info();

        // Should return a non-empty string
        assert!(!info.is_empty());

        // Should either list tools or say "No build tools found"
        let has_tools_info = info.contains("MSVC")
            || info.contains("MinGW")
            || info.contains("GCC")
            || info.contains("Xcode")
            || info.contains("No build tools found");

        assert!(
            has_tools_info,
            "Info should contain tool names or 'No build tools found'"
        );
    }

    /// Test check_build_tools returns appropriate result
    #[test]
    fn test_check_build_tools() {
        let result = py2pyd::check_build_tools();

        // Result should be Ok if tools exist, Err otherwise
        match result {
            Ok(tools) => {
                // If Ok, should have at least one tool
                assert!(
                    tools.has_any_tools(),
                    "If check_build_tools returns Ok, should have tools"
                );
            }
            Err(e) => {
                // If Err, error message should contain installation instructions
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("build tools") || error_msg.contains("install"),
                    "Error should mention build tools or installation"
                );
            }
        }
    }

    /// Test verify_build_tools (library API wrapper)
    #[test]
    fn test_verify_build_tools() {
        let result = py2pyd::verify_build_tools();

        // Same behavior as check_build_tools
        match result {
            Ok(tools) => {
                let info = tools.get_tools_info();
                assert!(!info.is_empty());
            }
            Err(_) => {
                // Expected on systems without build tools
            }
        }
    }

    /// Test BuildTools boolean methods consistency
    #[test]
    fn test_build_tools_consistency() {
        let tools = py2pyd::detect_build_tools();

        // has_any_tools should be true if any individual check is true
        let any_individual =
            tools.has_msvc() || tools.has_mingw() || tools.has_gcc() || tools.has_xcode();

        assert_eq!(
            tools.has_any_tools(),
            any_individual,
            "has_any_tools should match OR of individual checks"
        );
    }

    /// Test that MSVC detection works on Windows
    #[test]
    #[cfg(windows)]
    fn test_msvc_detection_windows() {
        let tools = py2pyd::detect_build_tools();

        // On Windows, we might have MSVC
        // This test just verifies the detection doesn't panic
        let _ = tools.has_msvc();

        if tools.has_msvc() {
            let info = tools.get_tools_info();
            assert!(
                info.contains("MSVC"),
                "Info should mention MSVC if detected"
            );
        }
    }

    /// Test that MinGW detection works on Windows
    #[test]
    #[cfg(windows)]
    fn test_mingw_detection_windows() {
        let tools = py2pyd::detect_build_tools();

        // On Windows, we might have MinGW
        let _ = tools.has_mingw();

        if tools.has_mingw() {
            let info = tools.get_tools_info();
            assert!(
                info.contains("MinGW"),
                "Info should mention MinGW if detected"
            );
        }
    }

    /// Test that GCC detection works on Unix
    #[test]
    #[cfg(unix)]
    fn test_gcc_detection_unix() {
        let tools = py2pyd::detect_build_tools();

        // On Unix, we might have GCC
        let _ = tools.has_gcc();

        if tools.has_gcc() {
            let info = tools.get_tools_info();
            assert!(info.contains("GCC"), "Info should mention GCC if detected");
        }
    }

    /// Test that Xcode detection works on macOS
    #[test]
    #[cfg(target_os = "macos")]
    fn test_xcode_detection_macos() {
        let tools = py2pyd::detect_build_tools();

        // On macOS, we might have Xcode
        let _ = tools.has_xcode();

        if tools.has_xcode() {
            let info = tools.get_tools_info();
            assert!(
                info.contains("Xcode"),
                "Info should mention Xcode if detected"
            );
        }
    }

    /// Test info string format
    #[test]
    fn test_info_string_format() {
        let tools = py2pyd::detect_build_tools();
        let info = tools.get_tools_info();

        // Info should not have leading/trailing whitespace issues
        // (though it might have trailing newlines which is fine)
        assert!(
            !info.starts_with('\n'),
            "Info should not start with newline"
        );

        // If tools are found, info should contain paths (with : or \)
        if tools.has_any_tools() {
            let has_path_separator =
                info.contains(':') || info.contains('\\') || info.contains('/');
            assert!(has_path_separator, "Tool info should contain file paths");
        }
    }

    /// Test that multiple calls return consistent results
    #[test]
    fn test_detection_consistency() {
        let tools1 = py2pyd::detect_build_tools();
        let tools2 = py2pyd::detect_build_tools();

        // Results should be consistent
        assert_eq!(tools1.has_msvc(), tools2.has_msvc());
        assert_eq!(tools1.has_mingw(), tools2.has_mingw());
        assert_eq!(tools1.has_gcc(), tools2.has_gcc());
        assert_eq!(tools1.has_xcode(), tools2.has_xcode());
        assert_eq!(tools1.has_any_tools(), tools2.has_any_tools());
    }
}
