import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletInvestmentsComponent } from './wallet-investments.component';

describe('WalletInvestmentsComponent', () => {
  let component: WalletInvestmentsComponent;
  let fixture: ComponentFixture<WalletInvestmentsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletInvestmentsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletInvestmentsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
