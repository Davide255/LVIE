#[macro_export]
macro_rules! build_operation {
    ($(($ty:ident, $args_type:ty),)*) => {
        use paste::paste;
        #[derive(Debug, Clone, PartialEq, Copy)]
        pub enum OperationType {
            $(
                $ty,
            )*
        }
        $(
            paste! {
                #[derive(Debug, Clone)]
                pub struct [<$ty Operation>] {
                    args: $args_type,
                    s_type: OperationType
                }

                impl Operation for [<$ty Operation>] {
                    fn get_type(&self) -> &OperationType {
                        &self.s_type
                    }
                }

                impl [<$ty Operation>] {
                    pub fn new(args: $args_type) -> Self {
                        [<$ty Operation>] {
                            args,
                            s_type: OperationType::$ty
                        }
                    }

                    pub fn get_content(&self) -> &$args_type {
                        &self.args
                    }
                }
            }
        )*

        impl History {
            $(
                paste! {
                    pub fn [<register_ $ty _Operation_without_saving>](&mut self, args: &$args_type) {
                        self.start_from_now();
                        self.history.push(
                            (false, Box::new([<$ty Operation>]::new(args.clone())))
                        );
                    }
                }

                paste! {
                    pub fn [<register_ $ty _Operation_and_save>]<P>(
                        &mut self,
                        args: &$args_type,
                        buffer: &image::ImageBuffer<P, Vec<P::Subpixel>>,
                    ) -> std::io::Result<()>
                    where
                        P: image::Pixel + std::fmt::Debug,
                        P::Subpixel: image::Primitive + std::fmt::Debug + num_traits::ToBytes + bytemuck::Pod,
                    {
                        self.start_from_now();
                        self.history
                            .push((true, Box::new([<$ty Operation>]::new(args.clone()))));
                        if self.use_temp_files { self.file_handler.write(buffer)?; }
                        Ok(())
                    }
                }
            )*
        }
    };
}
