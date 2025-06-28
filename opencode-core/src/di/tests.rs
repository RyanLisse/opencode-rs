//! Tests for dependency injection container
//! 
//! This test suite defines the expected behavior for the DI container
//! and service registration/resolution following TDD principles.

use super::*;
use std::sync::Arc;
use async_trait::async_trait;

#[cfg(test)]
mod di_container_tests {
    use super::*;

    #[test]
    fn test_container_creation() {
        // GIVEN: A new DI container
        let container = Container::new();
        
        // WHEN: We check its initial state
        // THEN: It should be empty
        assert_eq!(container.service_count(), 0);
    }

    #[test]
    fn test_singleton_registration_and_resolution() {
        // GIVEN: A container with a singleton service
        let mut container = Container::new();
        
        // Define a test service
        #[derive(Clone)]
        struct TestService {
            value: String,
        }
        
        impl TestService {
            fn new() -> Self {
                Self {
                    value: "test".to_string(),
                }
            }
        }
        
        // WHEN: We register and resolve the service
        container.register_singleton::<TestService>(|| Arc::new(TestService::new()));
        
        let service1 = container.resolve::<TestService>().unwrap();
        let service2 = container.resolve::<TestService>().unwrap();
        
        // THEN: Both resolutions should return the same instance
        assert!(Arc::ptr_eq(&service1, &service2));
        assert_eq!(service1.value, "test");
    }

    #[test]
    fn test_factory_registration_and_resolution() {
        // GIVEN: A container with a factory service
        let mut container = Container::new();
        
        // Counter to track factory invocations
        let counter = Arc::new(std::sync::Mutex::new(0));
        let counter_clone = counter.clone();
        
        #[derive(Clone)]
        struct FactoryService {
            id: u32,
        }
        
        // WHEN: We register a factory
        container.register_factory::<FactoryService>(move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            Arc::new(FactoryService { id: *count })
        });
        
        let service1 = container.resolve::<FactoryService>().unwrap();
        let service2 = container.resolve::<FactoryService>().unwrap();
        
        // THEN: Each resolution should create a new instance
        assert!(!Arc::ptr_eq(&service1, &service2));
        assert_eq!(service1.id, 1);
        assert_eq!(service2.id, 2);
    }

    #[test]
    fn test_interface_registration() {
        // GIVEN: An interface and multiple implementations
        trait Database: Send + Sync {
            fn name(&self) -> &str;
        }
        
        struct PostgresDB;
        impl Database for PostgresDB {
            fn name(&self) -> &str {
                "PostgreSQL"
            }
        }
        
        struct MySQLDB;
        impl Database for MySQLDB {
            fn name(&self) -> &str {
                "MySQL"
            }
        }
        
        // WHEN: We register implementations for the interface
        let mut container = Container::new();
        
        container.register_interface::<dyn Database, PostgresDB>(
            "postgres",
            || Arc::new(PostgresDB),
        );
        
        container.register_interface::<dyn Database, MySQLDB>(
            "mysql",
            || Arc::new(MySQLDB),
        );
        
        // THEN: We can resolve specific implementations
        let postgres = container.resolve_interface::<dyn Database>("postgres").unwrap();
        let mysql = container.resolve_interface::<dyn Database>("mysql").unwrap();
        
        assert_eq!(postgres.name(), "PostgreSQL");
        assert_eq!(mysql.name(), "MySQL");
    }

    #[test]
    fn test_dependency_injection_with_dependencies() {
        // GIVEN: Services with dependencies
        #[derive(Clone)]
        struct ConfigService {
            api_key: String,
        }
        
        #[derive(Clone)]
        struct ApiClient {
            config: Arc<ConfigService>,
        }
        
        impl ApiClient {
            fn new(config: Arc<ConfigService>) -> Self {
                Self { config }
            }
        }
        
        #[derive(Clone)]
        struct UserService {
            api_client: Arc<ApiClient>,
        }
        
        impl UserService {
            fn new(api_client: Arc<ApiClient>) -> Self {
                Self { api_client }
            }
        }
        
        // WHEN: We register services with dependencies
        let mut container = Container::new();
        
        container.register_singleton::<ConfigService>(|| {
            Arc::new(ConfigService {
                api_key: "secret123".to_string(),
            })
        });
        
        container.register_singleton_with_deps::<ApiClient, (Arc<ConfigService>,)>(
            |deps| {
                let (config,) = deps;
                Arc::new(ApiClient::new(config))
            }
        );
        
        container.register_singleton_with_deps::<UserService, (Arc<ApiClient>,)>(
            |deps| {
                let (api_client,) = deps;
                Arc::new(UserService::new(api_client))
            }
        );
        
        // THEN: Dependencies should be resolved correctly
        let user_service = container.resolve::<UserService>().unwrap();
        assert_eq!(user_service.api_client.config.api_key, "secret123");
    }

    #[test]
    fn test_scoped_services() {
        // GIVEN: A container with scoped services
        let mut container = Container::new();
        
        #[derive(Clone)]
        struct RequestContext {
            request_id: String,
        }
        
        // WHEN: We register a scoped service
        container.register_scoped::<RequestContext>();
        
        // Create scope 1
        let mut scope1 = container.create_scope();
        scope1.provide::<RequestContext>(Arc::new(RequestContext {
            request_id: "req-123".to_string(),
        }));
        
        // Create scope 2
        let mut scope2 = container.create_scope();
        scope2.provide::<RequestContext>(Arc::new(RequestContext {
            request_id: "req-456".to_string(),
        }));
        
        // THEN: Each scope should have its own instance
        let ctx1 = scope1.resolve::<RequestContext>().unwrap();
        let ctx2 = scope2.resolve::<RequestContext>().unwrap();
        
        assert_eq!(ctx1.request_id, "req-123");
        assert_eq!(ctx2.request_id, "req-456");
    }

    #[test]
    fn test_circular_dependency_detection() {
        // GIVEN: Services with circular dependencies
        let mut container = Container::new();
        
        // This should be detected and handled appropriately
        // Implementation would need cycle detection
    }

    #[test]
    fn test_service_not_found() {
        // GIVEN: A container without a specific service
        let container = Container::new();
        
        struct UnregisteredService;
        
        // WHEN: We try to resolve an unregistered service
        let result = container.resolve::<UnregisteredService>();
        
        // THEN: It should return an error
        assert!(result.is_err());
        match result {
            Err(DIError::ServiceNotFound(type_name)) => {
                assert!(type_name.contains("UnregisteredService"));
            }
            _ => panic!("Expected ServiceNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_async_initialization() {
        // GIVEN: Services that require async initialization
        #[derive(Clone)]
        struct AsyncService {
            data: String,
        }
        
        impl AsyncService {
            async fn new() -> Self {
                // Simulate async initialization
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                Self {
                    data: "async initialized".to_string(),
                }
            }
        }
        
        // WHEN: We register an async service
        let mut container = Container::new();
        
        container.register_async_singleton::<AsyncService>(|| {
            Box::pin(async {
                Arc::new(AsyncService::new().await)
            })
        });
        
        // THEN: We should be able to resolve it
        let service = container.resolve_async::<AsyncService>().await.unwrap();
        assert_eq!(service.data, "async initialized");
    }

    #[test]
    fn test_service_lifetime_management() {
        // GIVEN: Services with different lifetimes
        let mut container = Container::new();
        
        // Track service creation
        let singleton_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let transient_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        
        let singleton_count_clone = singleton_count.clone();
        let transient_count_clone = transient_count.clone();
        
        #[derive(Clone)]
        struct SingletonService {
            id: u32,
        }
        
        #[derive(Clone)]
        struct TransientService {
            id: u32,
        }
        
        // Register singleton
        container.register_singleton::<SingletonService>(move || {
            let id = singleton_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Arc::new(SingletonService { id })
        });
        
        // Register transient
        container.register_factory::<TransientService>(move || {
            let id = transient_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Arc::new(TransientService { id })
        });
        
        // WHEN: We resolve services multiple times
        let singleton1 = container.resolve::<SingletonService>().unwrap();
        let singleton2 = container.resolve::<SingletonService>().unwrap();
        let transient1 = container.resolve::<TransientService>().unwrap();
        let transient2 = container.resolve::<TransientService>().unwrap();
        
        // THEN: Singleton should be created once, transient multiple times
        assert_eq!(singleton1.id, 0);
        assert_eq!(singleton2.id, 0);
        assert_eq!(transient1.id, 0);
        assert_eq!(transient2.id, 1);
        
        assert_eq!(singleton_count.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(transient_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn test_container_builder_pattern() {
        // GIVEN: A container builder
        let container = ContainerBuilder::new()
            .register_singleton::<ConfigService>(|| {
                Arc::new(ConfigService {
                    api_key: "test-key".to_string(),
                })
            })
            .register_factory::<RequestContext>(|| {
                Arc::new(RequestContext {
                    request_id: uuid::Uuid::new_v4().to_string(),
                })
            })
            .build();
        
        // WHEN: We use the built container
        let config = container.resolve::<ConfigService>().unwrap();
        let ctx1 = container.resolve::<RequestContext>().unwrap();
        let ctx2 = container.resolve::<RequestContext>().unwrap();
        
        // THEN: Services should be properly registered
        assert_eq!(config.api_key, "test-key");
        assert_ne!(ctx1.request_id, ctx2.request_id); // Factory creates new instances
    }
}

// Type definitions that will be moved to the actual implementation
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub struct Container {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    factories: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    interfaces: HashMap<(TypeId, String), Box<dyn Any + Send + Sync>>,
    scoped_types: HashMap<TypeId, ()>,
}

pub struct Scope<'a> {
    container: &'a Container,
    scoped_instances: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

pub struct ContainerBuilder {
    container: Container,
}

#[derive(Debug, thiserror::Error)]
pub enum DIError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Circular dependency detected")]
    CircularDependency,
    
    #[error("Service already registered: {0}")]
    AlreadyRegistered(String),
    
    #[error("Invalid service lifetime")]
    InvalidLifetime,
}

// Placeholder implementations
impl Container {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            factories: HashMap::new(),
            interfaces: HashMap::new(),
            scoped_types: HashMap::new(),
        }
    }
    
    pub fn service_count(&self) -> usize {
        self.services.len() + self.factories.len()
    }
    
    pub fn register_singleton<T: Any + Send + Sync + 'static>(
        &mut self,
        factory: impl Fn() -> Arc<T> + Send + Sync + 'static,
    ) {
        let service = factory();
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }
    
    pub fn register_factory<T: Any + Send + Sync + 'static>(
        &mut self,
        factory: impl Fn() -> Arc<T> + Send + Sync + 'static,
    ) {
        self.factories.insert(TypeId::of::<T>(), Box::new(factory));
    }
    
    pub fn register_interface<I: ?Sized + 'static, T: I + Send + Sync + 'static>(
        &mut self,
        name: &str,
        factory: impl Fn() -> Arc<T> + Send + Sync + 'static,
    ) {
        let service = factory();
        self.interfaces.insert(
            (TypeId::of::<I>(), name.to_string()),
            Box::new(service as Arc<I>),
        );
    }
    
    pub fn register_singleton_with_deps<T: Any + Send + Sync + 'static, D>(
        &mut self,
        factory: impl Fn(D) -> Arc<T> + Send + Sync + 'static,
    ) where
        D: ResolveDependencies,
    {
        // Implementation would resolve dependencies and call factory
    }
    
    pub fn register_scoped<T: Any + Send + Sync + 'static>(&mut self) {
        self.scoped_types.insert(TypeId::of::<T>(), ());
    }
    
    pub fn register_async_singleton<T: Any + Send + Sync + 'static>(
        &mut self,
        factory: impl Fn() -> Pin<Box<dyn Future<Output = Arc<T>> + Send>> + Send + Sync + 'static,
    ) {
        // Implementation would store async factory
    }
    
    pub fn resolve<T: Any + Send + Sync + 'static>(&self) -> Result<Arc<T>, DIError> {
        // Try singletons first
        if let Some(service) = self.services.get(&TypeId::of::<T>()) {
            if let Some(arc) = service.downcast_ref::<Arc<T>>() {
                return Ok(arc.clone());
            }
        }
        
        // Try factories
        if let Some(factory) = self.factories.get(&TypeId::of::<T>()) {
            if let Some(f) = factory.downcast_ref::<Box<dyn Fn() -> Arc<T> + Send + Sync>>() {
                return Ok(f());
            }
        }
        
        Err(DIError::ServiceNotFound(std::any::type_name::<T>().to_string()))
    }
    
    pub fn resolve_interface<I: ?Sized + 'static>(
        &self,
        name: &str,
    ) -> Result<Arc<I>, DIError> {
        if let Some(service) = self.interfaces.get(&(TypeId::of::<I>(), name.to_string())) {
            if let Some(arc) = service.downcast_ref::<Arc<I>>() {
                return Ok(arc.clone());
            }
        }
        
        Err(DIError::ServiceNotFound(format!("{} ({})", std::any::type_name::<I>(), name)))
    }
    
    pub async fn resolve_async<T: Any + Send + Sync + 'static>(&self) -> Result<Arc<T>, DIError> {
        // Implementation would handle async resolution
        self.resolve::<T>()
    }
    
    pub fn create_scope(&self) -> Scope {
        Scope {
            container: self,
            scoped_instances: HashMap::new(),
        }
    }
}

impl<'a> Scope<'a> {
    pub fn provide<T: Any + Send + Sync + 'static>(&mut self, instance: Arc<T>) {
        self.scoped_instances.insert(TypeId::of::<T>(), Box::new(instance));
    }
    
    pub fn resolve<T: Any + Send + Sync + 'static>(&self) -> Result<Arc<T>, DIError> {
        // Check scoped instances first
        if let Some(instance) = self.scoped_instances.get(&TypeId::of::<T>()) {
            if let Some(arc) = instance.downcast_ref::<Arc<T>>() {
                return Ok(arc.clone());
            }
        }
        
        // Fall back to container
        self.container.resolve::<T>()
    }
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self {
            container: Container::new(),
        }
    }
    
    pub fn register_singleton<T: Any + Send + Sync + 'static>(
        mut self,
        factory: impl Fn() -> Arc<T> + Send + Sync + 'static,
    ) -> Self {
        self.container.register_singleton(factory);
        self
    }
    
    pub fn register_factory<T: Any + Send + Sync + 'static>(
        mut self,
        factory: impl Fn() -> Arc<T> + Send + Sync + 'static,
    ) -> Self {
        self.container.register_factory(factory);
        self
    }
    
    pub fn build(self) -> Container {
        self.container
    }
}

// Trait for resolving dependencies
pub trait ResolveDependencies {
    fn resolve(container: &Container) -> Self;
}

// Implement for tuples of dependencies
impl<T1: Any + Send + Sync + 'static> ResolveDependencies for (Arc<T1>,) {
    fn resolve(container: &Container) -> Self {
        (container.resolve::<T1>().unwrap(),)
    }
}