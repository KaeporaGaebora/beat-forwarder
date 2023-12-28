# Beat Forwarder

Beat Forwarder acts as a bridge between OS2L and OSC applications such as Virtual DJ and QLC+. By intercepting the data 
before forwarding it out, it provides a proper compatibility interface, allowing such features as:
* Only sending every nth beat
* Providing feedback to Virtual DJ buttons so that they light up
* Changing beat modes according to VDJ buttons
* Setting mutually exclusive buttons that turn themselves off when others are turned on
