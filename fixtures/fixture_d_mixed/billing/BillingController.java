package com.example.billing;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
public class BillingController {
    @GetMapping("/api/v1/billing/summary")
    public String getBillingSummary() {
        return "ok";
    }
}
