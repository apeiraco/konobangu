macro_rules! register_system_task_type {
    (
        $(#[$type_meta:meta])*
        $task_vis:vis struct $task_name:ident {
            $($(#[$field_meta:meta])* pub $field_name:ident: $field_type:ty),* $(,)?
        }
    ) => {
        $(#[$type_meta])*
        #[derive(typed_builder::TypedBuilder, ts_rs::TS, serde::Serialize, serde::Deserialize)]
        #[ts(rename_all = "camelCase")]
        $task_vis struct $task_name {
            $($(#[$field_meta])* pub $field_name: $field_type,)*
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[builder(default = None)]
            pub subscriber_id: Option<i32>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[builder(default = None)]
            pub cron_id: Option<i32>,
        }

        paste::paste! {
            $(#[$type_meta])*
            #[derive(typed_builder::TypedBuilder, ts_rs::TS, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[ts(rename_all = "camelCase")]
            $task_vis struct [<$task_name Input>] {
                $($(#[$field_meta])* pub $field_name: $field_type,)*
                #[serde(default, skip_serializing_if = "Option::is_none")]
                pub subscriber_id: Option<i32>,
                #[serde(default, skip_serializing_if = "Option::is_none")]
                pub cron_id: Option<i32>,
            }
        }

        impl $crate::task::SystemTaskTrait for $task_name {
            paste::paste! {
                type InputType = [<$task_name Input>];
            }

            fn get_subscriber_id(&self) -> Option<i32> {
                self.subscriber_id
            }

            fn get_cron_id(&self) -> Option<i32> {
                self.cron_id
            }

            fn set_subscriber_id(&mut self, subscriber_id: Option<i32>) {
                self.subscriber_id = subscriber_id;
            }

            fn set_cron_id(&mut self, cron_id: Option<i32>) {
                self.cron_id = cron_id;
            }

            fn from_input(input: Self::InputType, subscriber_id: Option<i32>) -> Self {
                Self {
                    $($field_name: input.$field_name,)*
                    subscriber_id: input.subscriber_id.or(subscriber_id),
                    cron_id: input.cron_id,
                }
            }
        }
    }
}

pub(crate) use register_system_task_type;
