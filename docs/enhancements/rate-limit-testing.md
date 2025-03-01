Below is a nicely formatted outline of the rate limiting test implementation:

---

### Test Setup

1. **Test Infrastructure**  
   Use the existing test infrastructure:

   - `common::test_with_postgres`
   - `common::test_with_redis`

2. **Test Application Instance**  
   Create a test app instance using default options.

3. **Test User Account**  
   Create a test user account (using `common::create_user`).

---

### Test Case Implementation

1. **Sending Failed Login Requests**

   - Send 5 consecutive failed login requests with an incorrect password.
   - Verify that each of these requests returns an HTTP **401 Unauthorized** status.

2. **Exceeding the Rate Limit**

   - Issue a 6th login request.
   - Verify that the response has an HTTP **429 Too Many Requests** status.
   - Confirm that the error response body matches the format:  
     `{"error": "Rate limit exceeded..."}`

3. **Optional: Testing Rate Limit Window Expiration**
   - Wait for window_secs+1 seconds to allow the rate limit window to expire.
   - Verify that subsequent login requests work as expected (i.e., they no longer trigger the rate limit).

---

### Test Organization

1. **File Structure**

   - Create a new module named `login_rate_limiting` in `users.rs`.

2. **Test Function**

   - Add a new test function within the module.
   - Annotate the test function with `#[tokio::test]` to run it asynchronously.

3. **Helper Functions and Test Request Patterns**
   - Leverage existing helper functions like `common::create_user` for user setup.
   - Use the established test request patterns for consistency.

---

### Summary

This test will verify that the system correctly applies rate limiting for consecutive failed login attempts as follows:

- Five failed login attempts return a **401 Unauthorized** status.
- The sixth attempt returns an HTTP **429 Too Many Requests** with the correct error message.
- Optionally, after waiting 61 seconds, normal login behavior resumes.

---
