//! 错误处理中间件单元测试

use backend::middleware::{ApiError, ErrorResponse};

#[cfg(test)]
mod error_handler_tests {
    use super::*;

    #[test]
    fn test_api_error_construction() {
        let err = ApiError::unauthorized("Test message");
        assert_eq!(err.error_type(), "unauthorized");
        assert_eq!(err.status_code(), 401);
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::not_found("User not found");
        assert!(format!("{}", err).contains("User not found"));
    }

    #[test]
    fn test_all_error_types() {
        // Unauthorized
        assert_eq!(ApiError::unauthorized("msg").error_type(), "unauthorized");
        assert_eq!(ApiError::unauthorized("msg").status_code(), 401);

        // Forbidden
        assert_eq!(ApiError::forbidden("msg").error_type(), "forbidden");
        assert_eq!(ApiError::forbidden("msg").status_code(), 403);

        // NotFound
        assert_eq!(ApiError::not_found("msg").error_type(), "not_found");
        assert_eq!(ApiError::not_found("msg").status_code(), 404);

        // BadRequest
        assert_eq!(ApiError::bad_request("msg").error_type(), "bad_request");
        assert_eq!(ApiError::bad_request("msg").status_code(), 400);

        // InternalError
        assert_eq!(ApiError::internal("msg").error_type(), "internal_error");
        assert_eq!(ApiError::internal("msg").status_code(), 500);

        // Conflict
        assert_eq!(ApiError::conflict("msg").error_type(), "conflict");
        assert_eq!(ApiError::conflict("msg").status_code(), 409);

        // ValidationError
        assert_eq!(ApiError::validation("msg").error_type(), "validation_error");
        assert_eq!(ApiError::validation("msg").status_code(), 400);
    }

    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse::new("not_found", "User not found".to_string(), 404);
        assert_eq!(response.error, "not_found");
        assert_eq!(response.message, "User not found");
        assert_eq!(response.status, 404);
        assert!(response.details.is_none());
    }

    #[test]
    fn test_error_response_with_details() {
        let response = ErrorResponse::new("validation_error", "Invalid input".to_string(), 400);
        let response_with_details = response.with_details(serde_json::json!({
            "field": "email",
            "reason": "Invalid format"
        }));

        assert_eq!(response_with_details.error, "validation_error");
        assert!(response_with_details.details.is_some());
    }

    #[test]
    fn test_convenience_constructors() {
        // Test all convenience constructors
        assert_eq!(ApiError::unauthorized("").error_type(), "unauthorized");
        assert_eq!(ApiError::forbidden("").error_type(), "forbidden");
        assert_eq!(ApiError::not_found("").error_type(), "not_found");
        assert_eq!(ApiError::bad_request("").error_type(), "bad_request");
        assert_eq!(ApiError::internal("").error_type(), "internal_error");
        assert_eq!(ApiError::conflict("").error_type(), "conflict");
        assert_eq!(ApiError::payload_too_large("").error_type(), "payload_too_large");
        assert_eq!(ApiError::unsupported_media_type("").error_type(), "unsupported_media_type");
        assert_eq!(ApiError::service_unavailable("").error_type(), "service_unavailable");
        assert_eq!(ApiError::database("").error_type(), "database_error");
        assert_eq!(ApiError::validation("").error_type(), "validation_error");
    }

    #[test]
    fn test_error_display_format() {
        let err = ApiError::unauthorized("Invalid token");
        let display = format!("{}", err);
        assert_eq!(display, "Unauthorized: Invalid token");
    }

    #[test]
    fn test_error_status_codes() {
        // 401
        assert_eq!(ApiError::unauthorized("").status_code().as_u16(), 401);

        // 403
        assert_eq!(ApiError::forbidden("").status_code().as_u16(), 403);

        // 404
        assert_eq!(ApiError::not_found("").status_code().as_u16(), 404);

        // 400
        assert_eq!(ApiError::bad_request("").status_code().as_u16(), 400);
        assert_eq!(ApiError::validation("").status_code().as_u16(), 400);

        // 500
        assert_eq!(ApiError::internal("").status_code().as_u16(), 500);
        assert_eq!(ApiError::database("").status_code().as_u16(), 500);

        // 409
        assert_eq!(ApiError::conflict("").status_code().as_u16(), 409);

        // 413
        assert_eq!(ApiError::payload_too_large("").status_code().as_u16(), 413);

        // 415
        assert_eq!(ApiError::unsupported_media_type("").status_code().as_u16(), 415);

        // 503
        assert_eq!(ApiError::service_unavailable("").status_code().as_u16(), 503);
    }

    #[test]
    fn test_error_message_content() {
        let err = ApiError::not_found("User with ID 123 not found");
        let message = format!("{}", err);
        assert!(message.contains("User with ID 123 not found"));
    }

    #[test]
    fn test_multiple_errors_different_types() {
        let errors = vec![
            ApiError::unauthorized("No token"),
            ApiError::forbidden("Admin only"),
            ApiError::not_found("User not found"),
            ApiError::bad_request("Invalid email"),
            ApiError::internal("Database connection failed"),
        ];

        let types: Vec<&str> = errors.iter().map(|e| e.error_type()).collect();
        assert!(types.contains(&"unauthorized"));
        assert!(types.contains(&"forbidden"));
        assert!(types.contains(&"not_found"));
        assert!(types.contains(&"bad_request"));
        assert!(types.contains(&"internal_error"));
    }
}
