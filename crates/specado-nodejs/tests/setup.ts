/**
 * Test setup and global configuration
 */

// Extend Jest matchers
expect.extend({
  toBeValidSpecadoError(received: any) {
    const pass = received && 
                 typeof received.kind === 'string' &&
                 typeof received.message === 'string' &&
                 typeof received.code === 'string';
    
    if (pass) {
      return {
        message: () => `expected ${received} not to be a valid SpecadoError`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${received} to be a valid SpecadoError with kind, message, and code properties`,
        pass: false,
      };
    }
  },
});

// Global test configuration
process.env.NODE_ENV = 'test';

// Increase timeout for async operations
jest.setTimeout(30000);

declare global {
  namespace jest {
    interface Matchers<R> {
      toBeValidSpecadoError(): R;
    }
  }
}